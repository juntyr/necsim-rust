FROM gitpod/workspace-full

USER gitpod

RUN wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/cuda-ubuntu1804.pin && \
    sudo mv cuda-ubuntu1804.pin /etc/apt/preferences.d/cuda-repository-pin-600 && \
    sudo apt-key adv --fetch-keys https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/7fa2af80.pub && \
    sudo add-apt-repository "deb https://developer.download.nvidia.com/compute/cuda/repos/ubuntu1804/x86_64/ /" && \
    sudo apt-get update -q && \
    sudo apt-get -y install cuda

RUN sudo apt-get update -q && \
    sudo apt-get upgrade -y && \
    sudo apt-get install gdal-bin -y --no-install-recommends && \
    sudo apt-get install libgdal-dev -y --no-install-recommends && \
    sudo apt-get clean autoclean && \
    sudo apt-get autoremove -y && \
    sudo rm -rf /var/lib/{apt,dpkg,cache,log}/
