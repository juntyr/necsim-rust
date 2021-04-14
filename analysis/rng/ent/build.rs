#![deny(clippy::pedantic)]

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=ent");

    std::process::Command::new("make").status().map(|_| ())
}
