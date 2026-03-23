FROM romhack-base

# gcc-mipsel-linux-gnu is the correct toolchain for PS1 decomp work.
# The PS1 (MIPS R3000A) runs in little-endian mode; use -nostdlib and
# -ffreestanding in your project to target bare-metal.
RUN apt install -y \
    binutils-mipsel-linux-gnu \
    gcc-mipsel-linux-gnu

# NOTE: PSn00bSDK (homebrew SDK) requires a bare-metal mipsel-none-elf
# toolchain, which is not available in Ubuntu apt and must be built from
# source. See: https://github.com/Lameguy64/PSn00bSDK
# To add PSn00bSDK support, extend this image via Dockerfile.local.

RUN useradd -m -d /ps1hax ps1hax

USER ps1hax

WORKDIR /ps1hax

RUN XDG_BIN_HOME=/ps1hax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
