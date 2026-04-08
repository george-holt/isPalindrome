# Full polyglot toolchain on the official devcontainer base. No Dev Container Features (no GHCR):
# everything is apt, NodeSource, Microsoft .NET repo, or rustup — same pattern as a hand-built Linux CI image.

FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

USER root
ENV DEBIAN_FRONTEND=noninteractive

# C/C++ toolchains for Bazel cc_* (Python 3.12 is default `python3` on Ubuntu 24.04)
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    lcov \
    pkg-config \
    python3 \
    python3-pip \
    python3-venv \
    git \
    ca-certificates \
    curl \
    wget \
    xdg-utils \
    gnupg \
    python3-coverage \
    && ln -sf /usr/bin/python3 /usr/local/bin/python \
    && rm -rf /var/lib/apt/lists/*

# Hermetic Bazel LLVM clang links libtinfo.so.5; Ubuntu 24.04 provides libtinfo6 only.
RUN arch="$(dpkg --print-architecture)" \
    && case "$arch" in \
         amd64) tinfo_url="http://archive.ubuntu.com/ubuntu/pool/universe/n/ncurses/libtinfo5_6.3-2ubuntu0.1_amd64.deb" ;; \
         arm64) tinfo_url="http://ports.ubuntu.com/ubuntu-ports/pool/universe/n/ncurses/libtinfo5_6.3-2ubuntu0.1_arm64.deb" ;; \
         *) echo "unsupported arch for libtinfo5: ${arch}" >&2; exit 1 ;; \
       esac \
    && wget -q -O /tmp/libtinfo5.deb "${tinfo_url}" \
    && apt-get update \
    && apt-get install -y /tmp/libtinfo5.deb \
    && rm -f /tmp/libtinfo5.deb \
    && rm -rf /var/lib/apt/lists/*

# Node.js 22 (NodeSource)
RUN curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && corepack enable \
    && rm -rf /var/lib/apt/lists/*

# .NET 8 SDK (packages.microsoft.com)
RUN wget -q https://packages.microsoft.com/config/ubuntu/24.04/packages-microsoft-prod.deb -O /tmp/packages-microsoft-prod.deb \
    && dpkg -i /tmp/packages-microsoft-prod.deb \
    && apt-get update \
    && apt-get install -y dotnet-sdk-8.0 \
    && rm -rf /var/lib/apt/lists/* /tmp/packages-microsoft-prod.deb

# Bazelisk as `bazel` (repo root `.bazelversion` selects the Bazel release).
ARG BAZELISK_VERSION=1.28.1
RUN arch="$(uname -m)" \
    && case "$arch" in \
         x86_64) bazel_arch=amd64 ;; \
         aarch64) bazel_arch=arm64 ;; \
         *) echo "unsupported arch for bazelisk: $arch" >&2; exit 1 ;; \
       esac \
    && curl -fsSL -o /usr/local/bin/bazel \
         "https://github.com/bazelbuild/bazelisk/releases/download/v${BAZELISK_VERSION}/bazelisk-linux-${bazel_arch}" \
    && chmod 755 /usr/local/bin/bazel

USER vscode
WORKDIR /home/vscode

ENV RUSTUP_HOME=/home/vscode/.rustup
ENV CARGO_HOME=/home/vscode/.cargo
ENV PATH=/home/vscode/.cargo/bin:${PATH}

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

RUN dotnet tool install -g dotnet-reportgenerator-globaltool \
    && dotnet tool install -g dotnet-coverage

ENV PATH=/home/vscode/.dotnet/tools:${PATH}

WORKDIR /workspaces
