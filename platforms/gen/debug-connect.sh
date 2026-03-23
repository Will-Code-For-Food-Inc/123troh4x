#!/usr/bin/env bash
# Connect gdb-multiarch to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 1234
#
# BlastEm has full 68000 register support in its GDB stub — this is a
# real, functional debug path (not an approximation like NES/SNES).
#
# Emulator examples:
#   BlastEm: launch with the -D flag to enable the GDB stub on port 1234.
#            Example: blastem -D rom.bin

HOST=${1:-host.docker.internal}
PORT=${2:-1234}

gdb-multiarch \
    -ex "set architecture m68k" \
    -ex "target remote $HOST:$PORT"
