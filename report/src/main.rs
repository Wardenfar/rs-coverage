#![feature(stdin_forwarders)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use anyhow::private::kind::TraitKind;
use proc_macro2::{Ident, Span};
use syn::spanned::Spanned;
use syn::{ItemFn, Stmt};
use syn::visit::{Visit, visit_block, visit_stmt};

fn main() -> anyhow::Result<()> {
    let mut functions: HashMap<String, HashMap<String, bool>> = HashMap::new();
    let mut files: HashMap<String, HashMap<u64, u64>> = HashMap::new();

    for line in std::io::stdin().lines() {
        let line = line?;
        if line.starts_with("[cov] ") {
            let line = &line[6..];
            let mut parts = line.splitn(2, " ");
            let line: u64 = parts.next().unwrap().parse()?;
            let file = parts.next().unwrap();
            *files.entry(file.into()).or_default().entry(line).or_default() += 1;
        }
    }

    for (file, lines) in &mut files {
        let info = file_info(&file)?;
        let mut fun = HashMap::new();

        for f in info.functions {
            let mut passed = false;
            for line in f.lines {
                if lines.contains_key(&line) {
                    passed = true;
                } else {
                    lines.insert(line, 0);
                }
            }
            fun.insert(f.name, passed);
        }

        functions.insert(file.clone(), fun);
    }

    let mut f = File::create("cov.info")?;

    writeln!(&mut f, "TN:")?;
    for (file, lines) in files {
        if !lines.is_empty() {
            writeln!(f, "SF:{}", file.replace("\\", "/"))?;
            let functions = functions.get(&file).unwrap();
            for (name, passed) in functions {
                let val = if *passed { 1 } else { 0 };
                writeln!(f, "FN:{},{}", val, name)?;
                writeln!(f, "FNDA:{},{}", val, name)?;
            }
            writeln!(f, "FNF:{}", functions.len())?;
            writeln!(f, "FNH:0")?;
            writeln!(f, "BRF:0")?;
            writeln!(f, "BRH:0")?;
            let len = lines.len();

            let mut lines: Vec<_> = lines.into_iter().collect();
            lines.sort_by_key(|k| k.0);
            for (line, count) in lines.into_iter() {
                writeln!(f, "DA:{},{}", line, count)?;
            }
            writeln!(f, "LF:{}", len)?;
            writeln!(f, "LH:0")?;
            writeln!(f, "end_of_record")?;
        }
    }

    Ok(())
}

struct Function {
    name: String,
    lines: HashSet<u64>
}

#[derive(Default)]
struct FileInfo {
    functions: Vec<Function>,
}

#[derive(Default)]
struct Stmts {
    not_leaf: bool,
    lines: HashSet<u64>
}

impl<'ast> Visit<'ast> for Stmts {
    fn visit_stmt(&mut self, i: &'ast Stmt) {
        let mut stmts = Stmts::default();
        visit_stmt(&mut stmts, i);

        if stmts.not_leaf {
            self.not_leaf = true;
            self.lines.extend(stmts.lines)
        } else if stmts.lines.is_empty() {
            self.not_leaf = true;
            let start = i.span().start().line as u64;
            let end = i.span().end().line as u64;
            self.lines.extend(start..=end)
        }
    }
}

impl<'ast> Visit<'ast> for FileInfo {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let name = i.sig.ident.to_string();

        let mut stmts = Stmts::default();
        visit_block(&mut stmts, &i.block);

        if stmts.lines.is_empty() {
            return;
        }

        self.functions.push(Function {
            name,
            lines: stmts.lines
        })
    }
}

fn file_info(path: &str) -> anyhow::Result<FileInfo> {
    let content = std::fs::read_to_string(path)?;
    let file = syn::parse_file(&*content)?;
    let mut info = FileInfo::default();
    syn::visit::visit_file(&mut info, &file);
    Ok(info)
}