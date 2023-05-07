FROM ubuntu:latest
LABEL maintainer='emulator000@gmail.com'

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
    lsb-release curl build-essential libssl-dev libclang-dev pkg-config \
    python3 python3-dev python3-pip wget unzip git && \
    pip3 install numpy && \
    pip3 install torch && \
    pip3 install diffusers && \
    pip3 install transformers && \
    pip3 install omegaconf && \
    pip3 install safetensors && \
    pip3 install torchvision && \
    pip3 install torchaudio && \
    pip3 install accelerate && \
    pip3 install pillow && \
    apt-get clean all

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN mkdir -p /libtorch && wget https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-2.0.0%2Bcpu.zip \
    && unzip libtorch-cxx11-abi-shared-with-deps-2.0.0+cpu.zip -d /

ENV LIBTORCH=/libtorch
ENV LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
