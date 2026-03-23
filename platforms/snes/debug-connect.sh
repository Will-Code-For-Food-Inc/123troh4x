#!/usr/bin/env bash
# Connect gdb-multiarch to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 2345
#
# The 65816 is not a native GDB architecture. The stub negotiates the
# architecture description; most SNES decomp teams use Mesen's built-in
# debugger for step/break workflows rather than GDB.
#
# Emulator examples:
#   Mesen 2: Debugger > GDB Integration (host side).
#            bsnes and snes9x do not expose GDB stubs.

HOST=${1:-host.docker.internal}
PORT=${2:-2345}

gdb-multiarch \
    -ex "target remote $HOST:$PORT"
