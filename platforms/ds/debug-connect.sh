#!/usr/bin/env bash
# Connect arm-none-eabi-gdb to a GDB stub running on the host.
#
# Usage: ./debug-connect.sh [HOST] [PORT]
#   HOST  defaults to host.docker.internal
#   PORT  defaults to 2345
#
# Emulator examples:
#   mGBA:     mgba -g 2345 rom.gba
#   DeSmuME:  desmume --arm9gdb=2345 rom.nds
#   SameBoy:  launch with GDB stub enabled on the configured port

HOST=${1:-host.docker.internal}
PORT=${2:-2345}

arm-none-eabi-gdb -ex "target remote $HOST:$PORT"
