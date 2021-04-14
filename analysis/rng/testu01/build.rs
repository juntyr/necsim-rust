#![deny(clippy::pedantic)]

fn main() {
    println!("cargo:rerun-if-changed=src/main.c");
    println!("cargo:rerun-if-changed=testu01");
}
