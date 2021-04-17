#![deny(clippy::pedantic)]

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=src/main.c");
    println!("cargo:rerun-if-changed=include/util64bits32bits.h");

    println!("cargo:rerun-if-changed=testu01");

    std::process::Command::new("make").status().map(|_| ())
}
