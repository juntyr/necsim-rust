if [ ! -f "$(dirname "$0")/target/release/rustcoalescence-linker" ]; then
    cargo build --release --target-dir "$(dirname "$0")/target" --manifest-path "$(dirname "$0")/../rustcoalescence/linker/Cargo.toml"
fi

"$(dirname "$0")/target/release/rustcoalescence-linker" "$@"
