FROM romhack-base

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpng-dev \
    zlib1g-dev

ARG RGBDS_TAG=v0.9.1
RUN git clone --depth 1 --branch ${RGBDS_TAG} https://github.com/gbdev/rgbds.git /tmp/rgbds \
    && cmake -S /tmp/rgbds -B /tmp/rgbds/build -DCMAKE_BUILD_TYPE=Release \
    && cmake --build /tmp/rgbds/build -j$(nproc) \
    && cmake --install /tmp/rgbds/build \
    && rm -rf /tmp/rgbds

RUN useradd -m -d /gbchax gbchax

USER gbchax

WORKDIR /gbchax

RUN XDG_BIN_HOME=/gbchax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
