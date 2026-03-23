FROM romhack-base

RUN dpkg --add-architecture i386 && apt update && apt install -y \
    gcc-arm-none-eabi \
    libc6:i386 \
    nasm

RUN useradd -m -d /dshax dshax

USER dshax

WORKDIR /dshax

RUN XDG_BIN_HOME=/dshax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
