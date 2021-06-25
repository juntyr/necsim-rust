cd $(dirname "$0")

if [ ! -f target/release/rustcoalescence-linker ]; then
    cargo build --release
fi

target/release/rustcoalescence-linker "$@"
