# Use the official Microsoft devcontainers base image
FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential pkg-config libudev-dev libssl-dev curl git \
    && rm -rf /var/lib/apt/lists/*

# Switch to vscode user
USER vscode
WORKDIR /home/vscode

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Set up PATH for Rust
ENV PATH="/home/vscode/.cargo/bin:$PATH"

# Install Solana CLI v1.16.25 via pre-built binary
RUN mkdir -p ~/.local/share && \
    curl -L https://github.com/solana-labs/solana/releases/download/v1.16.25/solana-release-x86_64-unknown-linux-gnu.tar.bz2 | tar -jx && \
    mv solana-release ~/.local/share/solana

# Install AVM (Anchor Version Manager)
RUN /home/vscode/.cargo/bin/cargo install --git https://github.com/coral-xyz/anchor avm --force

# Install and use Anchor 0.28.0
RUN /home/vscode/.cargo/bin/avm install 0.28.0
RUN /home/vscode/.cargo/bin/avm use 0.28.0

# Rollback to Rustc 1.68.0
RUN rustup override set 1.68.0

# Set up PATH for all tools
ENV PATH="/home/vscode/.cargo/bin:/home/vscode/.local/share/solana/bin:/home/vscode/.avm/bin:$PATH"

# Create workspace directory
RUN mkdir -p /home/vscode/workspace
WORKDIR /home/vscode/workspace

# Default command
CMD ["/bin/bash"] 