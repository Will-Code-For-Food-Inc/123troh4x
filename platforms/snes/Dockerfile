FROM romhack-base

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpng-dev \
    zlib1g-dev

ARG ASAR_TAG=v1.91
RUN git clone --depth 1 --branch ${ASAR_TAG} https://github.com/RPGHacker/asar.git /tmp/asar \
    && cmake -S /tmp/asar/src -B /tmp/asar/build \
        -DCMAKE_BUILD_TYPE=Release \
        -DASAR_BUILD_DLL=OFF \
        -DASAR_BUILD_TESTS=OFF \
    && cmake --build /tmp/asar/build -j$(nproc) \
    && cmake --install /tmp/asar/build \
    && rm -rf /tmp/asar

RUN useradd -m -d /sneshax sneshax

USER sneshax

WORKDIR /sneshax

RUN XDG_BIN_HOME=/sneshax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
