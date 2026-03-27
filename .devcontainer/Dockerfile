# Full polyglot toolchain on the official devcontainer base. No Dev Container Features (no GHCR):
# everything is apt, NodeSource, Microsoft .NET repo, or rustup — same pattern as a hand-built Linux CI image.

FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

USER root
ENV DEBIAN_FRONTEND=noninteractive

# C/C++ + CMake (cpp/ requires CMake >= 3.24; noble ships 3.28.x)
# Python 3.12 is default `python3` on Ubuntu 24.04
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    ninja-build \
    pkg-config \
    python3 \
    python3-pip \
    python3-venv \
    git \
    ca-certificates \
    curl \
    wget \
    gnupg \
    && ln -sf /usr/bin/python3 /usr/local/bin/python \
    && rm -rf /var/lib/apt/lists/*

# Node.js 22 (NodeSource)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/*

# .NET 8 SDK (packages.microsoft.com)
RUN wget -q https://packages.microsoft.com/config/ubuntu/24.04/packages-microsoft-prod.deb -O /tmp/packages-microsoft-prod.deb \
    && dpkg -i /tmp/packages-microsoft-prod.deb \
    && apt-get update \
    && apt-get install -y dotnet-sdk-8.0 \
    && rm -rf /var/lib/apt/lists/* /tmp/packages-microsoft-prod.deb

USER vscode
WORKDIR /home/vscode

ENV RUSTUP_HOME=/home/vscode/.rustup
ENV CARGO_HOME=/home/vscode/.cargo
ENV PATH=/home/vscode/.cargo/bin:${PATH}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

WORKDIR /workspaces
