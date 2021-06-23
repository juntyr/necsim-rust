cargo run --release --target-dir "$(dirname "$0")/target" --manifest-path "$(dirname "$0")/../rustcoalescence/linker/Cargo.toml" -- "$@"
