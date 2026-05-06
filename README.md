# ics-cli

Local-markdown workspace + Stratum **ICS** (Incremental Consensus System) integration. This repository is the home for the **`ics` CLI**, implemented in **Rust** (Cargo package **`ics-cli`**); the running server and canonical ICS behavior live in **[Stratum](https://github.com/Stratum-ICS/Stratum)**.

## Install

| Method | When to use |
|--------|-------------|
| **[GitHub Releases](https://github.com/Stratum-ICS/ics-cli/releases)** | Download a **prebuilt** archive (`ics-*-x86_64-unknown-linux-gnu.tar.gz`, macOS `.tar.gz`, or Windows `.zip`) for your platform. Extract and put `ics` on your `PATH`. |
| **Cargo from git** | `cargo install --locked --git https://github.com/Stratum-ICS/ics-cli --branch master` |
| **Build from source** | Clone the repo, then `cargo build --release` → `target/release/ics` |

Release binaries are built by [`.github/workflows/release.yml`](.github/workflows/release.yml) when you **publish** a [GitHub Release](https://github.com/Stratum-ICS/ics-cli/releases) for a tag.

## Docs

| Document | Description |
|----------|-------------|
| [**Usage wiki**](docs/wiki/README.md) | Commands, config, Stratum flows, storage, troubleshooting |
| [Design spec](docs/superpowers/specs/2026-05-06-ics-cli-design.md) | Phased plan (B0–B2–C), hybrid `.ics/` layout, Stratum API mapping, identity bridge |
| [B0 plan](docs/superpowers/plans/2026-05-05-ics-cli-b0.md) | Local store + `ics init/commit/log/diff/status` |
| [B1 plan](docs/superpowers/plans/2026-05-05-ics-cli-b1.md) | Stratum auth, notes, proposals |
| [B2 plan](docs/superpowers/plans/2026-05-05-ics-cli-b2.md) | Pull server state, conflict flags |
| [C plan](docs/superpowers/plans/2026-05-05-ics-cli-c.md) | Vault `pull` preview/confirm |
| [Obsidian ICS plugin spec](docs/superpowers/specs/2026-05-05-obsidian-ics-plugin-spec.md) | Feature brief: phases P0–P4, non-goals, UX |
| [Obsidian ICS plugin design](docs/superpowers/specs/2026-05-05-obsidian-ics-plugin-design.md) | Approved architecture + **§11 user walkthrough** (ribbon, panel, modals, flows) |
| [Obsidian ICS plugin P0 plan](docs/superpowers/plans/2026-05-05-obsidian-ics-plugin-p0.md) | Sibling-repo implementation: esbuild, runner, `ItemView`, palette commands |

## Stratum references

- Wiki: [Teams & Proposals (ICS)](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/teams-proposals.md), [API reference](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/api-reference.md)
- In-repo survey (Stratum tree): [ICS and CLI-related tooling](https://github.com/Stratum-ICS/Stratum/blob/master/docs/superpowers/specs/2026-05-05-ics-cli-tool-repository-documentation.md)

## Status

Rust **`ics`** binary ships from this repo (`cargo build`, `cargo test`). HTTP paths are pinned via `tests/fixtures/stratum-openapi.json` — refresh when your Stratum OpenAPI changes.
