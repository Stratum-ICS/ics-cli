# Obsidian ICS plugin — feature spec

**Date:** 2026-05-05  
**Status:** Draft (separate deliverable from core `ics` CLI)  
**Parent context:** [ics-cli design spec](2026-05-06-ics-cli-design.md) — vault as markdown workspace, `.ics/` local store, Stratum for ICS and (later) vault HTTP.

---

## 1. Purpose

Deliver an **Obsidian Community Plugin** that turns a vault into the **editing UI** for the same workflows the **`ics`** CLI implements: local snapshots (B0+), Stratum-backed notes and proposals (B1+), pull and conflict surfacing (B2+), optional vault pull alignment (C). The plugin **does not** replace the CLI’s storage or API client logic; it **invokes and presents** `ics` (and may add thin UI-only helpers).

**Repository placement (TBD):** default assumption is a **sibling repo** (e.g. `obsidian-ics` or `ics-obsidian`) published separately from `ics-cli`, with this spec living in `ics-cli` until that repo exists or the team moves the canonical spec.

---

## 2. Non-goals (v0)

- Reimplementing `.ics/` hybrid storage, SQLite commit graph, or Stratum HTTP in TypeScript.
- ActivityPub / federation.
- Replacing server-side embeddings or `conflict_hints` computation.

---

## 3. Relation to CLI milestones

| Plugin phase | CLI milestone | Behavior |
|--------------|---------------|----------|
| **P0** | B0 | Settings; spawn `ics` for `status`, `commit`, `log`, `diff`; show output in notices / simple view. |
| **P1** | B0 | Optional dirty-state awareness (e.g. after save); diff in split or modal. |
| **P2** | B1 | Login / publish / proposal commands mapped to CLI subcommands; display read-only `conflict_hints` when returned. |
| **P3** | B2 | Pull / refresh; surface conflict policy per CLI flags/subcommands. |
| **P4** | C | Vault pull preview vs confirm, aligned with `POST /api/vaults/{slug}/pull` semantics via CLI. |

If a CLI subcommand is missing, the plugin phase **blocks** or degrades with a clear notice — no duplicated network client for parity-critical paths.

---

## 4. Architecture

### 4.1 Process bridge

- From plugin `main.ts`, **`spawn`** the `ics` binary with **vault root as `cwd`** (or pass an explicit root flag if the CLI adds one).
- Stream **stdout/stderr** into a dedicated **ICS** view or modal; map non-zero exit to `Notice` errors with trimmed stderr.
- Long-running commands must not block the Electron UI thread.

### 4.2 Settings (normative names — adjust to match shipped plugin)

| Setting | Purpose |
|---------|---------|
| `icsBinaryPath` | Absolute path or `ics` on `PATH` (Flatpak users may need `/usr/bin/ics` or `~/.cargo/bin/ics`). |
| `stratumBaseUrl` | Optional override; else inherit env / CLI default (`STRATUM_BASE_URL`). |
| `autoCheckStatus` | Optional: debounced `ics status` after file save (P1). |

**Secrets:** Prefer CLI-owned **`~/.config/ics-cli/credentials.json`** (mode `0600`); the plugin does not persist tokens in `data.json` unless a later spec explicitly requires it.

### 4.3 Identity bridge

Follow the parent spec: **YAML frontmatter** (`stratum_note_id`, etc.) and/or **path → id index** maintained by the CLI. The plugin may offer “refresh frontmatter from CLI” when the CLI exposes a stable **machine-readable** command (e.g. `ics note info --json`).

---

## 5. UX surface (minimal)

- **Ribbon** icon + **Command palette** entries for: Status, Commit (prompt for message), Log, Diff (current file or vault per CLI).
- **B1+:** Login, Publish, Proposal create/submit (forms: team, rationale ≥ 50 chars per Stratum rules).
- **B2+:** Pull / refresh; conflict indicators per CLI policy.

---

## 6. Implementation notes

- **Stack:** TypeScript, `manifest.json`, `versions.json`; build via `esbuild` or Obsidian sample plugin template.
- **Obsidian API:** declare a tested **minimum** `app` version.
- **Flatpak Obsidian:** home-directory vaults and spawning host `ics` are in scope; document sandbox pitfalls if users store vaults outside allowed mounts.

---

## 7. Stratum references

Same as parent spec: [teams & proposals](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/teams-proposals.md), [API reference](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/api-reference.md), [vault layout](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/vault-layout.md).

---

## 8. Open questions

- Canonical **plugin repo name** and whether this file moves there with a stub link from `ics-cli`.
- Whether **P1** hooks require new CLI flags (`--porcelain`, JSON lines) for stable parsing.
- **Community Plugins** listing criteria (license, README, minimal docs) before first publish.

---

## 9. Self-review

- **Scope:** UI + process bridge only; CLI remains source of truth for ICS and local history.
- **Coupling:** Explicit phase table prevents shipping plugin features ahead of CLI capabilities without duplication.
