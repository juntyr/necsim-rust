rustup default nightly
rustup toolchain install nightly -c rustfmt -c clippy -c rls -c rust-analysis -t nvptx64-nvidia-cuda --allow-downgrade || true

export CUDADIR="/usr/local/cuda"
export CUDAPATH="${CUDADIR}/lib64:${CUDADIR}/lib"
export PATH="${CUDADIR}/bin:${PATH}"
export LD_LIBRARY_PATH="${CUDAPATH}"
export LIBRARY_PATH="${LD_LIBRARY_PATH}"
