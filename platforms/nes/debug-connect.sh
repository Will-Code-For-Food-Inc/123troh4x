#!/usr/bin/env bash
# Connect gdb-multiarch to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 2345
#
# The 6502 is not a native GDB architecture. The i8086 target provides a
# workable approximation for memory inspection; register names will not
# correspond to 6502 registers. Most NES decomp teams use Mesen's built-in
# debugger for step/break workflows instead.
#
# Emulator examples:
#   Mesen 2: Debugger > GDB Integration, or --gdbport 2345 on the command line.
#            FCEUX does not expose a GDB stub.

HOST=${1:-host.docker.internal}
PORT=${2:-2345}

gdb-multiarch \
    -ex "set architecture i8086" \
    -ex "target remote $HOST:$PORT"
