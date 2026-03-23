FROM romhack-base

RUN apt install -y \
    binutils-m68k-linux-gnu \
    default-jre-headless \
    gcc-m68k-linux-gnu \
    sdcc

ARG SGDK_TAG=v2.00
RUN git clone --depth 1 --branch ${SGDK_TAG} https://github.com/Stephane-D/SGDK.git /opt/sgdk

ENV GDK=/opt/sgdk
ENV PATH="$PATH:/opt/sgdk/bin"

RUN useradd -m -d /genhax genhax

USER genhax

WORKDIR /genhax

RUN XDG_BIN_HOME=/genhax/.local/bin curl -LsSf https://astral.sh/uv/install.sh | sh

RUN sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)"

CMD ["zsh"]
