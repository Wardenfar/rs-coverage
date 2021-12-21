#![feature(proc_macro_span)]

extern crate proc_macro;

use proc_macro::{Span, TokenStream};

use quote::{quote, quote_spanned, ToTokens};
use syn::__private::TokenStream2;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::visit_mut::{visit_block_mut, VisitMut};
use syn::{
    parse_macro_input, parse_quote, Block, DeriveInput, Expr, ExprBlock, ExprIf, ItemFn, Stmt,
};

struct BlockVisitor;

impl VisitMut for BlockVisitor {
    fn visit_block_mut(&mut self, b: &mut Block) {
        visit_block_mut(self, b);
        let span = Span::call_site();
        let file = String::from(span.source_file().path().to_str().unwrap());
        let mut output: Vec<Stmt> = Vec::new();
        for s in b.stmts.drain(..) {
            let span = s.span();
            let cov: TokenStream2 = quote_spanned! { span =>
                println!("[cov] {} {}", line!(), #file);
            };
            let result = syn::parse(TokenStream::from(cov)).unwrap();
            output.push(result);
            output.push(s);
        }
        b.stmts = output;
    }
}

#[proc_macro_attribute]
pub fn cov(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item = parse_macro_input!(input as ItemFn);
    let mut visitor = BlockVisitor;
    BlockVisitor::visit_item_fn_mut(&mut visitor, &mut item);
    item.into_token_stream().into()
}
