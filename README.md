# 123troh4x

A multi-platform retro ROM hacking and homebrew development workspace. Each
platform has its own container with the appropriate toolchain. Containers are
build environments only — emulators run on the host.

## Platforms

| Platform | Directory | Toolchain | GDB stub emulator |
|----------|-----------|-----------|-------------------|
| NES | `platforms/nes/` | cc65 (ca65/ld65) | Mesen 2 |
| SNES | `platforms/snes/` | asar | Mesen 2 |
| Game Boy / GBC | `platforms/gbc/` | RGBDS, GBDK-2020 | SameBoy |
| Game Boy Advance | `platforms/gba/` | gcc-arm-none-eabi, agbcc | mGBA |
| Sega Genesis | `platforms/gen/` | gcc-m68k-linux-gnu, SGDK | BlastEm |
| Nintendo DS | `platforms/ds/` | gcc-arm-none-eabi, nasm | DeSmuME |
| Nintendo 64 | `platforms/n64/` | gcc-mips-linux-gnu, libdragon | *(none — see DEBUG-NOTES.md)* |
| PlayStation 1 | `platforms/ps1/` | gcc-mipsel-linux-gnu, PSn00bSDK | PCSX-Redux |

## Getting started

```bash
cd platforms/<platform>
./<platform>hax           # launch container
./<platform>hax --build   # build image and launch
```

## Debugging

Each platform includes a `debug-connect.sh` that connects the appropriate GDB
binary inside the container to a GDB stub exposed by an emulator on the host.
Start your emulator with its GDB stub enabled, then from inside the container:

```bash
./debug-connect.sh                        # use defaults
./debug-connect.sh host.docker.internal 2345
```

See each platform's `debug-connect.sh` for emulator-specific instructions.
N64 has no GDB stub — see `platforms/n64/DEBUG-NOTES.md`.

## Shared base

`shared/Dockerfile.base` provides a common Ubuntu layer (git, make, zsh, uv,
etc.) that platform Dockerfiles can build on.

Build it first if your platform Dockerfile references it:

```bash
docker build -t romhack-base ./shared/
```

## Personal customization

Containers are intentionally generic. Two patterns are available for layering
your own setup on top without touching the shared Dockerfiles.

### Dockerfile.local — bake tools into the image

For tools you always want present: a specific editor, custom shell, personal
utilities. Changes require rebuilding the derived image, but the result is
fully reproducible.

```bash
cp shared/Dockerfile.local.example platforms/ds/Dockerfile.local
# edit Dockerfile.local — FROM dshax:latest, then add your tools
make configure-ds     # builds dshax-local:latest
make run-ds-local     # launches your configured image
```

`Dockerfile.local` is gitignored — it stays on your machine.

### Dotfiles volume mount — mount configs at runtime

For dotfiles, shell config, editor settings. No rebuild needed; changes on
the host are reflected immediately on next launch.

```bash
make run-ds-configured                        # mounts ~/.dotfiles by default
make run-ds-configured DOTFILES=~/my-dotfiles # override the path
```

The dotfiles directory is mounted read-only at `/home/dshax/.dotfiles` (path
varies by platform). Run your install script manually from inside the container
on first launch.

### When to use which

| Situation | Pattern |
|-----------|---------|
| Installing an editor, compiler, or tool | `Dockerfile.local` |
| Dotfiles, shell config, editor settings | Volume mount |
| Trying something out temporarily | Volume mount |
| Sharing your setup with someone else | `Dockerfile.local` |

## Versioned tools

Several platforms build tools from source with pinnable version tags:

| Platform | Build arg | Tool |
|----------|-----------|------|
| GBC | `RGBDS_TAG` | RGBDS |
| SNES | `ASAR_TAG` | asar |
| Genesis | `SGDK_TAG` | SGDK |
| N64 | `LIBDRAGON_TAG` | libdragon toolchain |
| PS1 | *(PSn00bSDK requires a bare-metal mipsel-none-elf toolchain — see platforms/ps1/Dockerfile)* | — |

Override at build time:

```bash
docker build --build-arg ASAR_TAG=v1.91 platforms/snes/
```
