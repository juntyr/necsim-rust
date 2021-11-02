FROM gitpod/workspace-full

USER gitpod

ENV DEBIAN_FRONTEND=noninteractive

RUN echo "debconf debconf/frontend select Noninteractive" | sudo debconf-set-selections && \
    echo "keyboard-configuration keyboard-configuration/layout select 'English (US)'" | sudo debconf-set-selections && \
    echo "keyboard-configuration keyboard-configuration/layoutcode select 'us'" | sudo debconf-set-selections && \
    echo "resolvconf resolvconf/linkify-resolvconf boolean false" | sudo debconf-set-selections && \
    wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/cuda-ubuntu1804.pin && \
    sudo mv cuda-ubuntu1804.pin /etc/apt/preferences.d/cuda-repository-pin-600 && \
    sudo apt-key adv --fetch-keys https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/7fa2af80.pub && \
    sudo add-apt-repository "deb https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/ /" && \
    sudo apt-get update -q && \
    sudo apt-get install cuda -y --no-install-recommends

RUN sudo apt-get update -q && \
    sudo apt-get upgrade -y && \
    sudo apt-get install dieharder -y --no-install-recommends && \
    sudo apt-get install libopenmpi-dev -y --no-install-recommends && \
    sudo apt-get install gdal-bin -y --no-install-recommends && \
    sudo apt-get install libgdal-dev -y --no-install-recommends && \
    sudo apt-get clean autoclean && \
    sudo apt-get autoremove -y && \
    sudo rm -rf /var/lib/{apt,dpkg,cache,log}/

RUN cargo install ptx-linker --force --locked && \
    cargo install cargo-mpirun --force --locked && \
    cargo install cargo-reaper --git https://github.com/MomoLangenstein/grim-reaper --force --locked
