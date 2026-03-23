FROM romhack-base

RUN dpkg --add-architecture i386 && apt update && apt install -y \
    gcc-arm-none-eabi \
    libc6:i386 \
    libpng-dev

RUN git clone --depth 1 https://github.com/pret/agbcc.git /tmp/agbcc \
    && cd /tmp/agbcc \
    && ./build.sh \
    && ./install.sh /usr/local \
    && rm -rf /tmp/agbcc

ENV PATH="$PATH:/usr/local/tools/agbcc/bin"

RUN useradd -m -d /gbahax gbahax

USER gbahax

WORKDIR /gbahax

RUN XDG_BIN_HOME=/gbahax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
