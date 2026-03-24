PLATFORMS := nes snes gbc gba gen ds n64 ps1

CONTAINER_CMD := $(shell command -v podman 2>/dev/null || command -v docker 2>/dev/null)

# ─────────────────────────────────────────────────────────────────────────────
# Building
# ─────────────────────────────────────────────────────────────────────────────
# Build the shared base image first, then any platform image on top of it.
#
#   make build-base        — build shared/Dockerfile.base → romhack-base
#   make build-<platform>  — build a single platform image
#   make build-all         — build base, then all platform images
#
# Platform builds depend on build-base, so make handles the order automatically.
# Run `make -j build-all` to build all platform images in parallel after the base.

.PHONY: build-all build-base \
        $(addprefix build-,$(PLATFORMS)) \
        $(addprefix configure-,$(PLATFORMS)) \
        $(addprefix run-,$(PLATFORMS)) \
        $(addprefix run-,$(addsuffix -local,$(PLATFORMS))) \
        $(addprefix run-,$(addsuffix -configured,$(PLATFORMS)))

$(addprefix build-,$(PLATFORMS)): build-base

build-all: $(addprefix build-,$(PLATFORMS))

build-base:
	$(CONTAINER_CMD) build -t romhack-base -f ./shared/Dockerfile.base .

build-%:
	$(CONTAINER_CMD) build -t $*hax ./platforms/$*/


# ─────────────────────────────────────────────────────────────────────────────
# Running
# ─────────────────────────────────────────────────────────────────────────────
# Launch a platform container with your vendor/ directory mounted.
#
#   make run-<platform>   — run the standard platform image

run-%:
	$(CONTAINER_CMD) run -it \
		--userns=keep-id:uid=1001,gid=1001 \
		-v ./platforms/$*/vendor:/$*hax/vendor \
		$*hax:latest


# ─────────────────────────────────────────────────────────────────────────────
# Personal configuration — Dockerfile.local
# ─────────────────────────────────────────────────────────────────────────────
# For baking personal tools into a derived image: neovim, a custom shell
# config, your own compiler flags, etc.
#
# How it works:
#   1. Copy shared/Dockerfile.local.example to platforms/<platform>/Dockerfile.local
#   2. Edit it — FROM <platform>hax:latest, then add whatever you need
#   3. make configure-<platform>   — builds it as <platform>hax-local:latest
#   4. make run-<platform>-local   — launches the configured image
#
# Dockerfile.local is gitignored. Your personal setup stays out of the repo.

configure-%: build-%
	@if [ -f platforms/$*/Dockerfile.local ]; then \
		$(CONTAINER_CMD) build -t $*hax-local -f platforms/$*/Dockerfile.local platforms/$*/; \
	else \
		echo ""; \
		echo "  No Dockerfile.local found at platforms/$*/Dockerfile.local"; \
		echo "  Copy the example to get started:"; \
		echo "    cp shared/Dockerfile.local.example platforms/$*/Dockerfile.local"; \
		echo ""; \
		exit 1; \
	fi

run-%-local:
	$(CONTAINER_CMD) run -it \
		--userns=keep-id:uid=1001,gid=1001 \
		-v ./platforms/$*/vendor:/$*hax/vendor \
		$*hax-local:latest


# ─────────────────────────────────────────────────────────────────────────────
# Personal configuration — dotfiles volume mount
# ─────────────────────────────────────────────────────────────────────────────
# For mounting your dotfiles at runtime without rebuilding the image.
# No rebuild needed when your configs change — just re-run.
#
#   make run-<platform>-configured
#   make run-<platform>-configured DOTFILES=~/path/to/dotfiles
#
# DOTFILES defaults to ~/.dotfiles. The directory is mounted read-only at
# /home/<platform>hax/.dotfiles inside the container. Your dotfiles install
# script (if any) can be run manually from inside the container.

DOTFILES ?= $(HOME)/.dotfiles

run-%-configured:
	$(CONTAINER_CMD) run -it \
		--userns=keep-id:uid=1001,gid=1001 \
		-v ./platforms/$*/vendor:/$*hax/vendor \
		-v $(DOTFILES):/home/$*hax/.dotfiles:ro \
		$*hax:latest
