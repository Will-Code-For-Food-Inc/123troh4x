#!/usr/bin/env bash
# Connect gdb-multiarch to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 3333
#
# PCSX-Redux has a purpose-built GDB stub designed for decomp workflows.
#
# Emulator examples:
#   PCSX-Redux: Tools > GDB Server (default port 3333), or set "gdb": true
#               in the emulator's JSON config. Can also be toggled from the
#               Lua console: PCSX.setGdbServer(true, 3333)

HOST=${1:-host.docker.internal}
PORT=${2:-3333}

gdb-multiarch \
    -ex "set architecture mips:3000" \
    -ex "target remote $HOST:$PORT"
