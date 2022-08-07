#!/usr/bin/env bash

# set WASI_SDK_PATH to the correct location in your system

# sudo apt install libtinfo5
# wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-11/wasi-sdk_11.0_amd64_ubuntu20.04.deb
# sudo dpkg -i wasi-sdk_11.0_amd64_ubuntu20.04.deb
# rm wasi-sdk_11.0_amd64_ubuntu20.04.deb

export WASI_SDK_PATH="/opt/wasi-sdk"

export WASI_SYSROOT="${WASI_SDK_PATH}/share/wasi-sysroot"
export CC_wasm32_wasi="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SYSROOT}"
export AR_wasm32_wasi="${WASI_SDK_PATH}/bin/llvm-ar"

export BINDGEN_EXTRA_CLANG_ARGS_wasm32_wasi="--sysroot=${WASI_SYSROOT} -fvisibility=default"

export RUSTFLAGS="-C target-feature=-crt-static"

cargo build --release --target "wasm32-wasi"
