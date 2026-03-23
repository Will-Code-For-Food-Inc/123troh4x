#!/usr/bin/env bash
# Connect gdb-multiarch to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 2345
#
# The Game Boy CPU (Sharp SM83) is not a standard Z80 or AVR.
# gdb-multiarch with `set architecture sm83` is the correct approach.
# Note: avr-gdb and z80-elf-gdb target the wrong architectures.
#
# Emulator examples:
#   SameBoy: launch with GDB stub enabled (Debugger > Listen for GDB Connection),
#            configure the port to match PORT below.

HOST=${1:-host.docker.internal}
PORT=${2:-2345}

gdb-multiarch \
    -ex "set architecture sm83" \
    -ex "target remote $HOST:$PORT"
