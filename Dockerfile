FROM rust:1.84.0-bullseye AS builder
RUN apt-get update && apt-get -y upgrade && apt-get install -y cmake libclang-dev
RUN curl -L https://risczero.com/install | bash
ENV PATH="$PATH:/root/.risc0/bin"
RUN mkdir -p "$HOME/.cargo/bin"
RUN rzup install cargo-risczero 1.2.1
ENV PATH="$PATH:/root/.cargo/bin"
RUN cargo risczero install
COPY . lighthouse
ARG FEATURES
ARG PROFILE=release
ARG CARGO_USE_GIT_CLI=true
ENV FEATURES=$FEATURES
ENV PROFILE=$PROFILE
ENV CARGO_NET_GIT_FETCH_WITH_CLI=$CARGO_USE_GIT_CLI
RUN cd lighthouse && make


FROM ubuntu:22.04
RUN apt-get update && apt-get -y upgrade && apt-get install -y --no-install-recommends \
   curl \
   libssl-dev \
   ca-certificates \
   docker.io \
   && apt-get clean \
   && rm -rf /var/lib/apt/lists/*
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:$PATH"
RUN curl -L https://risczero.com/install | bash
ENV PATH="$PATH:/root/.risc0/bin"
RUN mkdir -p "$HOME/.cargo/bin"
RUN rzup install cargo-risczero 1.2.1
ENV PATH="$PATH:/root/.cargo/bin"
COPY --from=builder /usr/local/cargo/bin/lighthouse /usr/local/bin/lighthouse
RUN mkdir lighthouse
COPY --from=builder /lighthouse/target /lighthouse/target
ENV RISC0_WORK_DIR="/risc0workdir"
