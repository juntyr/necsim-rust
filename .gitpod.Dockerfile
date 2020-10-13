FROM gitpod/workspace-full

USER gitpod

RUN sudo apt-get update -q && \
    sudo apt-get upgrade -y && \
    sudo apt-get install gdal-bin -y --no-install-recommends && \
    sudo apt-get install libgdal-dev -y --no-install-recommends && \
    sudo apt-get clean autoclean && \
    sudo apt-get autoremove -y && \
    sudo rm -rf /var/lib/{apt,dpkg,cache,log}/
