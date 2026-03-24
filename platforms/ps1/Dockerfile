FROM romhack-base

# gcc-mipsel-linux-gnu is the correct toolchain for PS1 decomp work.
# The PS1 (MIPS R3000A) runs in little-endian mode; use -nostdlib and
# -ffreestanding in your project to target bare-metal.
RUN apt install -y \
    binutils-mipsel-linux-gnu \
    gcc-mipsel-linux-gnu \
    libgmp-dev \
    libmpc-dev \
    libmpfr-dev \
    texinfo

# Build mipsel-none-elf bare-metal toolchain required by PSn00bSDK.
# WARNING: This is a long build (~15-20 minutes).
# Uses binutils 2.40 + GCC 12.2.0, tested combination per PSn00bSDK docs.
ENV MIPSEL_NONE_ELF=/opt/mipsel-none-elf
ENV PATH="$PATH:$MIPSEL_NONE_ELF/bin"

RUN mkdir -p /tmp/toolchain/build-binutils /tmp/toolchain/build-gcc \
    && cd /tmp/toolchain \
    && wget -q https://ftp.gnu.org/gnu/binutils/binutils-2.40.tar.xz \
    && wget -q https://ftp.gnu.org/gnu/gcc/gcc-12.2.0/gcc-12.2.0.tar.xz \
    && tar -xf binutils-2.40.tar.xz \
    && tar -xf gcc-12.2.0.tar.xz \
    && cd /tmp/toolchain/build-binutils \
    && ../binutils-2.40/configure \
        --prefix=$MIPSEL_NONE_ELF \
        --target=mipsel-none-elf \
        --disable-docs \
        --disable-nls \
        --disable-werror \
        --with-float=soft \
    && make -j$(nproc) \
    && make install-strip \
    && cd /tmp/toolchain/build-gcc \
    && ../gcc-12.2.0/configure \
        --prefix=$MIPSEL_NONE_ELF \
        --target=mipsel-none-elf \
        --disable-docs \
        --disable-nls \
        --disable-werror \
        --disable-libada \
        --disable-libssp \
        --disable-libquadmath \
        --disable-threads \
        --disable-libgomp \
        --disable-libstdcxx-pch \
        --disable-hosted-libstdcxx \
        --enable-languages=c,c++ \
        --without-isl \
        --without-headers \
        --with-float=soft \
        --with-gnu-as \
        --with-gnu-ld \
    && make -j$(nproc) \
    && make install-strip \
    && rm -rf /tmp/toolchain

# Build PSn00bSDK against the bare-metal toolchain.
# Override PSN00BSDK_TAG at build time to pin a specific release:
#   docker build --build-arg PSN00BSDK_TAG=v0.24 .
# Check https://github.com/Lameguy64/PSn00bSDK/releases for the latest tag.
ARG PSN00BSDK_TAG=v0.24
ENV PSN00BSDK=/opt/psn00bsdk
RUN git clone --depth 1 --branch ${PSN00BSDK_TAG} --recurse-submodules \
        https://github.com/Lameguy64/PSn00bSDK.git /tmp/psn00bsdk \
    && cmake \
        --preset default \
        -DPSN00BSDK_TC=$MIPSEL_NONE_ELF \
        -DCMAKE_INSTALL_PREFIX=$PSN00BSDK \
        /tmp/psn00bsdk \
    && cmake --build /tmp/psn00bsdk/build \
    && cmake --install /tmp/psn00bsdk/build \
    && rm -rf /tmp/psn00bsdk

ENV PATH="$PATH:$PSN00BSDK/bin"

RUN useradd -m -d /ps1hax ps1hax

USER ps1hax

WORKDIR /ps1hax

RUN XDG_BIN_HOME=/ps1hax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
