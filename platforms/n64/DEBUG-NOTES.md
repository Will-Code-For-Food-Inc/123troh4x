# N64 Debugging Notes

No mainstream N64 emulator (mupen64plus, ares, Project64) exposes a GDB stub.
There is no `debug-connect.sh` for this platform.

`gdb-multiarch` is installed in the container for offline use — e.g. loading
an ELF for static analysis:

    gdb-multiarch ./build/game.elf

## Debugging options on the host

1. **ares built-in debugger** — integrated disassembler, memory viewer, and
   register inspector. No GDB remote protocol.

2. **mupen64plus Lua scripting** — hook into memory reads/writes and CPU
   events via the Lua API. Useful for tracing and automated testing.

3. **Hardware debugging** — USB64 and similar cartridge adapters give direct
   debug access on real N64 hardware via USB.

4. **Custom GDB patches** — some decomp teams (sm64, oot) have historically
   used patched mupen64plus builds with a GDB bridge. These are
   project-specific and not distributed as a generic tool.

If a mainstream emulator adds GDB stub support in the future, add a
`debug-connect.sh` following the pattern from other platforms and use:

    gdb-multiarch -ex "set architecture mips:4000" -ex "target remote HOST:PORT"
