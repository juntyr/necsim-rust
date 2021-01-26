rm -f ./.cargo/config.toml

cargo build --release --manifest-path necsim/linker/Cargo.toml

RUSTC_BOOTSTRAP=1 rustc -Z unstable-options --print target-spec-json \
    | python3 -c 'import json,sys;obj=json.load(sys.stdin);print('\
'"[target.{}]\nlinker = \"target/release/necsim-linker\""'\
'.format(obj["llvm-target"]))' \
    > ./.cargo/config.toml
