use coverage::cov;
use crate::b::test;

#[path = "a/b.rs"]
mod b;

#[cov]
fn main() {
    let a = 5;
    let b = if a > 2 {
        println!("test");
        test();
        2
    } else {
        unused();
        0
    };
    add(1, 5);
    add(10, 5);
    println!("{}", b);
}

#[cov]
fn add(a: usize, b:usize) -> usize {
    if a < b {
        a + b
    } else {
        0
    }
}

#[cov]
pub fn unused() {
    println!("tret")
}
