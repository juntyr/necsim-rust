rm -f ./.cargo/config.toml

cargo build --release --manifest-path rustcoalescence/linker/Cargo.toml

RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json \
    | python3 -c 'import json,sys;obj=json.load(sys.stdin);print('\
'"[target.{}]\nlinker = \"target/release/rustcoalescence-linker\""'\
'.format(obj["llvm-target"]))' \
    > ./.cargo/config.toml
