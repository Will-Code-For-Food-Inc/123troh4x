FROM romhack-base

RUN apt-get update && apt-get install -y --no-install-recommends \
    binutils \
    cc65

RUN useradd -m -d /neshax neshax

USER neshax

WORKDIR /neshax

RUN XDG_BIN_HOME=/neshax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
