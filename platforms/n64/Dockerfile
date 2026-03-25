FROM romhack-base

RUN apt-get update && apt-get install -y --no-install-recommends \
    binutils-mips-linux-gnu \
    gcc-mips-linux-gnu \
    libgmp-dev \
    libmpc-dev \
    libmpfr-dev \
    libpng-dev \
    texinfo

# Build and install the libdragon toolchain (mips64-elf GCC) for N64 homebrew.
# WARNING: This is a long build (~20-30 minutes).
# Override LIBDRAGON_TAG at build time:
#   docker build --build-arg LIBDRAGON_TAG=v8.0 .
# Check https://github.com/DragonMinded/libdragon/releases for the latest tag.
ARG LIBDRAGON_TAG=trunk
ENV N64_INST=/opt/n64chain
RUN git clone --depth 1 --branch ${LIBDRAGON_TAG} https://github.com/DragonMinded/libdragon.git /opt/libdragon \
    && cd /opt/libdragon \
    && ./tools/build-toolchain.sh \
    && rm -rf /opt/libdragon/tools/build

ENV LIBDRAGON=/opt/libdragon
ENV PATH="$PATH:/opt/n64chain/bin"

RUN useradd -m -d /n64hax n64hax

USER n64hax

WORKDIR /n64hax

RUN XDG_BIN_HOME=/n64hax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
