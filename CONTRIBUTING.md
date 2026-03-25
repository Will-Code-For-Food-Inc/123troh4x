# Contributing

Thanks for your interest. A few things to know before you start.

## This is a human-maintained project

Contributions are expected to come from people who own what they're submitting.
Automated agents acting without direct human involvement should not open PRs,
file issues, or post comments. See [agents.md](agents.md) for the full policy.

## Getting started

```sh
git clone --recurse-submodules https://github.com/Will-Code-For-Food-Inc/123troh4x
cd 123troh4x
make build-base        # build the shared base image (requires podman or docker)
make build-gba         # or whichever platform you're working on
cargo test --workspace # unit + binary integration tests (no container needed)
```

For full integration tests (requires a built platform image):

```sh
ROMHACK_INTEGRATION=1 ROMHACK_ROOT=$(pwd) cargo test --workspace
```

## Project layout

| Path | What it is |
|------|-----------|
| `shared/Dockerfile.base` | Shared Ubuntu base + `gami` binary |
| `platforms/<name>/` | Per-platform Dockerfile and toolchain |
| `protocol/` | Shared `Op`/`Request`/`Response` types |
| `tsukumogami/` | `gami` — in-container op runner |
| `onmyoji/` | MCP server (AI agent interface) |
| `knowledge/` | SQLite-backed ROM/symbol/annotation store |

## Workflow

- `main` is the stable branch. Work in a feature branch and open a PR.
- Keep PRs focused. One concern per PR.
- Tests must pass. Add tests for new behaviour.
- Run `cargo fmt` and `cargo clippy` before pushing.

## License

AGPL-3.0. By contributing you agree your changes will be licensed under the same terms.
