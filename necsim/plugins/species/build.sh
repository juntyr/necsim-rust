#!/usr/bin/env bash

# set WASI_SDK_PATH to the correct location in your system

export WASI_SDK_PATH="/opt/wasi-sdk"

export WASI_SYSROOT="${WASI_SDK_PATH}/share/wasi-sysroot"
export CC_wasm32_wasi="${WASI_SDK_PATH}/bin/clang --sysroot=${WASI_SYSROOT}"
export AR_wasm32_wasi="${WASI_SDK_PATH}/bin/llvm-ar"

export BINDGEN_EXTRA_CLANG_ARGS_wasm32_wasi="--sysroot=${WASI_SYSROOT} -fvisibility=default"

export LIBSQLITE3_FLAGS="\
    -DSQLITE_OS_OTHER \
    -USQLITE_TEMP_STORE \
    -DSQLITE_TEMP_STORE=3 \
    -USQLITE_THREADSAFE \
    -DSQLITE_THREADSAFE=0 \
    -DSQLITE_OMIT_LOCALTIME \
    -DSQLITE_OMIT_LOAD_EXTENSION \
    -DLONGDOUBLE_TYPE=double"

export RUSTFLAGS="-C target-feature=-crt-static"

cargo build --release --target "wasm32-wasi"
