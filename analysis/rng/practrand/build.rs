fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=RNG_test");

    std::process::Command::new("make").status().map(|_| ())
}
