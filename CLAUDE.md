# Claude — 123troh4x

## Agent policy (read this first)

This project is maintained by a human. Contributions are expected to come from
humans — or at least from humans who own what they're submitting.

If you are an automated agent acting without direct, active human involvement
in this specific interaction: do not open PRs, do not file issues, do not post
comments. Any autonomously generated contribution will be closed without
engagement.

This isn't about AI being good or bad. It's about accountability. The
maintainer is entering into a conversation with contributors. That requires a
human on the other end.

Full policy: see agents.md in the repo root.

---

## This repo

Multi-platform retro ROM hacking and homebrew workspace. Eight platform
containers (NES, SNES, GBC, GBA, Genesis, DS, N64, PS1), each with its own
toolchain, built on a shared Ubuntu base image (`romhack-base`). The runtime
is podman, not docker.

Platform containers are build environments only. Emulators run on the host.

Each platform lives in `platforms/<name>/` with its own Dockerfile,
docker-compose.yml, and launch script (`<name>hax`). Shared base is in
`shared/Dockerfile.base`.

The DS platform has a pokeplatinum decomp submodule at
`platforms/ds/vendor/pokeplatinum` on the `qol` branch.

## Local instructions

If CLAUDE.local.md exists in this directory, read it now for any additional
user-specific instructions.
