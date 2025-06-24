FROM rust:1.76

RUN apt-get update && \
    apt-get install -y pkg-config build-essential libssl-dev libudev-dev \
    git curl wget gnupg libclang-dev llvm clang lldb ca-certificates \
    libprotobuf-dev protobuf-compiler unzip

ENV SOLANA_VERSION=1.18.15
RUN sh -c "$(curl -sSfL https://release.anza.xyz/v${SOLANA_VERSION}/install)"
RUN echo 'export PATH="/root/.local/share/solana/install/active_release/bin:$PATH"' > /etc/profile.d/solana.sh

ENV ANCHOR_VERSION=0.29.0
RUN cargo install --git https://github.com/coral-xyz/anchor --tag v${ANCHOR_VERSION} anchor-cli --locked

WORKDIR /workspace