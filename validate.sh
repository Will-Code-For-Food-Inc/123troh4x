#!/usr/bin/env bash
set -e

pass() { echo "  [OK] $1"; }
fail() { echo "  [FAIL] $1"; }

check() {
    local image=$1
    local cmd=$2
    local label=$3
    if podman run --rm "$image" sh -c "$cmd" > /dev/null 2>&1; then
        pass "$label"
    else
        fail "$label"
    fi
}

echo ""
echo "=== NES ==="
check neshax "ca65 --version" "ca65"
check neshax "ld65 --version" "ld65"

echo ""
echo "=== SNES ==="
check sneshax "asar --version" "asar"

echo ""
echo "=== GBC ==="
check gbchax "rgbasm --version" "rgbasm"
check gbchax "rgblink --version" "rgblink"
check gbchax "rgbfix --version" "rgbfix"

echo ""
echo "=== GBA ==="
check gbahax "arm-none-eabi-gcc --version" "arm-none-eabi-gcc"
check gbahax "which agbcc" "agbcc"

echo ""
echo "=== Genesis ==="
check genhax "m68k-linux-gnu-gcc --version" "m68k-linux-gnu-gcc"
check genhax "test -d /opt/sgdk" "SGDK at /opt/sgdk"
check genhax "java --version" "java"

echo ""
echo "=== DS ==="
check dshax "arm-none-eabi-gcc --version" "arm-none-eabi-gcc"
check dshax "nasm --version" "nasm"

echo ""
echo "=== N64 ==="
check n64hax "mips-linux-gnu-gcc --version" "mips-linux-gnu-gcc"
check n64hax "test -d /opt/n64chain" "n64chain toolchain"
check n64hax "test -d /opt/libdragon" "libdragon"

echo ""
echo "=== PS1 ==="
if podman image exists ps1hax 2>/dev/null; then
    check ps1hax "mipsel-linux-gnu-gcc --version" "mipsel-linux-gnu-gcc"
else
    echo "  [SKIP] ps1hax not built yet"
fi

echo ""
