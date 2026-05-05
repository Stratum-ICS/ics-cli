# Obsidian ICS plugin — design

**Date:** 2026-05-05  
**Status:** Approved for planning (brainstorm: architecture §1–§4 locked)  
**Related:** [Feature spec](2026-05-05-obsidian-ics-plugin-spec.md) (phases, non-goals, UX bullets); parent [ics-cli design](2026-05-06-ics-cli-design.md).

---

## 1. Purpose & scope

Ship a **sibling-repo** Obsidian Community Plugin that uses the vault as the editing surface for **`ics`** workflows: local history (B0+), Stratum notes and proposals (B1+), pull and conflicts (B2+), optional vault pull (C). The plugin **invokes** the Rust CLI and **presents** output; it does **not** own `.ics/` storage, SQLite, or Stratum HTTP for parity-critical behavior.

---

## 2. Decisions

| Topic | Choice | Rationale |
|-------|--------|-----------|
| Source location | **Sibling** git repository | Clear release cycle vs `ics-cli`; this repo keeps links and docs until the plugin repo owns README/releases. |
| Integration | **Spawn `ics` only** (no bundled binary, no daemon in v0) | Single source of truth; matches feature spec; revisit bundling only if onboarding data demands it. |
| Secrets | **CLI config only** (`~/.config/ics-cli/credentials.json`, mode `0600`) | Aligns with parent ics-cli design; plugin settings do not store tokens. |

Plugin repository **name** is chosen at sibling-repo bootstrap (e.g. `ics-obsidian`); not blocking design sign-off.

---

## 3. Non-goals (v0)

Same as [feature spec §2](2026-05-05-obsidian-ics-plugin-spec.md): no reimplementation of storage or Stratum client in TypeScript; no federation; no server-side embedding replacement.

---

## 4. Plugin phases vs CLI milestones

| Plugin phase | CLI milestone | Behavior |
|--------------|---------------|----------|
| **P0** | B0 | Settings; spawn `ics` for `status`, `commit`, `log`, `diff`; output in dedicated surface + notices. |
| **P1** | B0 | Optional debounced `status` after save; diff in split or modal. |
| **P2** | B1 | Login, publish, proposal commands; read-only display of `conflict_hints` when CLI surfaces them. |
| **P3** | B2 | Pull / refresh; conflict policy per CLI. |
| **P4** | C | Vault pull preview / confirm via CLI. |

If a subcommand is missing, the feature **blocks** or degrades with a clear notice — **no** duplicate HTTP client for parity-critical paths.

---

## 5. Design — §1 Architecture

The sibling repo ships a standard Obsidian plugin (TypeScript, `manifest.json`). At runtime the plugin resolves **`icsBinaryPath`** (default `ics` on `PATH`) and uses **`vault.adapter.getBasePath()`** (or equivalent vault root) as **`cwd`** for every spawn. Optional **`STRATUM_BASE_URL`** is injected into the child environment when the user sets `stratumBaseUrl` in plugin settings. **Long-running** commands must not block the UI: use async spawn and stream handlers. **Credentials** are never read from plugin `data.json` in v0.

---

## 6. Design — §2 Components & data flow

| Unit | Responsibility | Depends on |
|------|----------------|------------|
| **`IcsPlugin` (`main.ts`)** | `onload` / `onunload`; ribbon, commands, settings tab; owns one **runner**. | Obsidian `Plugin` API |
| **Settings + settings tab** | `icsBinaryPath`, optional `stratumBaseUrl`, P1 options (`autoCheckStatus`, debounce ms). | `loadData` / `saveData` |
| **`IcsRunner`** | `child_process.spawn` with `cwd` = vault root, merged `env`; API such as `run(argv, { signal? })` streaming stdout/stderr with distinct prefixes. | Node `child_process` |
| **Output surface** | Dedicated workspace leaf or modal consuming runner stream. | `Workspace`, `ItemView` or modal API |
| **Command handlers** | Map palette → argv; optional preflight only if needed (otherwise rely on CLI errors). | Runner |

**Flow:** “ICS: Status” → `runner.run(['status'])` → stream to output leaf → exit `0` silent or short success notice; non-zero → §7.

---

## 7. Design — §3 Error handling

- **ENOENT / missing binary:** `Notice` naming configured path; deep-link or highlight Settings.  
- **Non-zero exit:** `Notice` with truncated combined output (~2 KB cap, suffix `…`). Do not log secrets; rely on CLI not echoing tokens.  
- **Cancel:** If wired, abort signal → SIGTERM on child, SIGKILL after short grace.  
- **Concurrency:** Serialize or queue spawns per vault so one output view does not interleave two sessions.  
- **Flatpak Obsidian:** Sibling repo README documents home-dir vaults and typical `ics` paths (`/usr/bin/ics`, `~/.cargo/bin/ics`); warn if vault lives outside Flatpak-exposed paths.

---

## 8. Design — §4 Testing

- **P0:** Manual checklist: vault with `ics init`, each palette command, failure cases (bad binary, dirty tree).  
- **Optional:** Pure argv/path helpers in `src/lib/` covered by **Vitest** (no Obsidian imports).  
- **Release:** Declare minimum Obsidian `app` version in `manifest.json`; re-run checklist on that version before Community Plugins submission.

---

## 9. Settings (normative intent)

| Key | Purpose |
|-----|---------|
| `icsBinaryPath` | Absolute path or `ics` on `PATH`. |
| `stratumBaseUrl` | Optional; sets `STRATUM_BASE_URL` for child. |
| `autoCheckStatus` | P1: debounced `ics status` after save. |

Exact key strings may match `camelCase` in shipped code.

---

## 10. Identity & Stratum references

Identity bridge (frontmatter / index) follows [ics-cli design §3.2](2026-05-06-ics-cli-design.md). Stratum wiki: [teams & proposals](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/teams-proposals.md), [API reference](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/api-reference.md), [vault layout](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/vault-layout.md).

---

## 11. Self-review (design doc)

- **Placeholders:** Only deferred item is sibling **repo name** at creation (explicit).  
- **Consistency:** Spawn-only + sibling repo matches §2 decisions; phase table matches feature spec.  
- **Scope:** One plugin product, one integration style; P4 deferred until CLI C.  
- **Ambiguity:** “Output leaf” may be `MarkdownView` or custom `ItemView` — implementation plan picks one for P0.
