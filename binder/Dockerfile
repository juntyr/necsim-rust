FROM python:3.9-slim

# install the notebook package
RUN pip install --no-cache --upgrade pip && \
    pip install --no-cache notebook

RUN apt-get update -q && \
    apt-get upgrade -y && \
    apt-get install curl -y --no-install-recommends && \
    apt-get clean autoclean && \
    apt-get autoremove -y && \
    rm -rf /var/lib/{apt,dpkg,cache,log}/

# create user with a home directory
ARG NB_USER
ARG NB_UID
ENV USER ${NB_USER}
ENV HOME /home/${NB_USER}

RUN adduser --disabled-password \
    --gecos "Default user" \
    --uid ${NB_UID} \
    ${NB_USER}
WORKDIR ${HOME}
USER ${USER}

SHELL ["/bin/bash", "-c"]

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
    sh -s -- --default-toolchain none --profile minimal -y && \
    source $HOME/.cargo/env && \
    ./setup.sh && \
    cargo build --release \
                --features rustcoalescence-algorithms-monolithic \
                --features rustcoalescence-algorithms-independent && \
    mv target/release/*.so . && \
    cargo install --path rustcoalescence --locked \
                  --features rustcoalescence-algorithms-monolithic \
                  --features rustcoalescence-algorithms-independent
