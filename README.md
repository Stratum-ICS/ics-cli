# ics-cli

Local-markdown workspace + Stratum **ICS** (Incremental Consensus System) integration. This repository is the home for the **`ics` CLI**, implemented in **Rust** (Cargo package **`ics-cli`**); the running server and canonical ICS behavior live in **[Stratum](https://github.com/Stratum-ICS/Stratum)**.

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

## Stratum references

- Wiki: [Teams & Proposals (ICS)](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/teams-proposals.md), [API reference](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/api-reference.md)
- In-repo survey (Stratum tree): [ICS and CLI-related tooling](https://github.com/Stratum-ICS/Stratum/blob/master/docs/superpowers/specs/2026-05-05-ics-cli-tool-repository-documentation.md)

## Status

Rust **`ics`** binary ships from this repo (`cargo build`, `cargo test`). HTTP paths are pinned via `tests/fixtures/stratum-openapi.json` — refresh when your Stratum OpenAPI changes.
