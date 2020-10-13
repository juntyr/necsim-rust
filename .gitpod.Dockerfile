FROM gitpod/workspace-full

USER gitpod

RUN sudo add-apt-repository ppa:ubuntugis/ppa -y && \
    sudo apt-get update -q && \
    sudo apt-get upgrade -y && \
    sudo apt-get install gdal-bin && \
    sudo apt-get install libgdal-dev && \
    sudo apt-get clean autoclean && \
    sudo apt-get autoremove -y && \
    sudo rm -rf /var/lib/{apt,dpkg,cache,log}/
