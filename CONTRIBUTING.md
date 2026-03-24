# Contributing

## The one rule

Contributions come from humans who own what they're submitting. Use whatever
tools you want to write code — but when you open a PR or file an issue, that's
you talking. Own it.

The full version is in [agents.md](agents.md). Read it.

## Repo structure

```
platforms/<name>/       # one directory per platform
    Dockerfile          # toolchain for this platform
    docker-compose.yml
    <name>hax           # launch script
    debug-connect.sh    # GDB stub helper
    vendor/             # submodules, local projects (gitignored contents)
    docs/               # platform-specific docs (DS has these, others TBD)

shared/
    Dockerfile.base     # common Ubuntu layer all platforms build on
    Dockerfile.local.example
```

Build the base image first, then any platform:

```bash
make base
make build-ds    # or whatever platform
```

## Adding a new platform

1. Create `platforms/<name>/` with a Dockerfile extending `romhack-base`
2. Add a `<name>hax` launch script (follow an existing one as a template)
3. Add a `docker-compose.yml` and `debug-connect.sh`
4. Add a target to the root Makefile
5. Add the platform to the table in README.md

## PRs

- Keep them focused. One thing per PR.
- If it's non-trivial, open an issue first so we can talk about it before you
  spend time on it.
- If I ask a follow-up question and get no response, the PR will eventually be
  closed. See the one rule above.

## Personal setup

Containers are intentionally generic. The README covers two patterns for
layering your own setup on top without touching shared files:
[Dockerfile.local](README.md#dockerfilelocal--bake-tools-into-the-image) for
baking tools in, and
[dotfiles volume mount](README.md#dotfiles-volume-mount--mount-configs-at-runtime)
for shell config and editor settings. Neither gets committed.
