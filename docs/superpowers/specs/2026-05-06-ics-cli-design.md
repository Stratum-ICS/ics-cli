# ics-cli — design spec

**Date:** 2026-05-06  
**Status:** Approved direction (local markdown first, vault integration later)  
**Backend of record:** [Stratum](https://github.com/Stratum-ICS/Stratum) — ICS via HTTP APIs; vault file sync via `/api/vaults/...` when milestone **C** is implemented.

---

## 1. Purpose

Build a CLI that treats a **local markdown workspace** as the primary editing surface (Obsidian-compatible layout optional), with **local history** that feels like a small, dedicated **git** for notes. **ICS** operations (teams, proposals, reviews) go through Stratum’s **REST API** once online. **Vault-centric** sync (`/api/vaults/...` pull/fork/clone semantics) is a **later** milestone when Stratum’s server-side vault story is the completeness bar we target.

This repo holds the CLI code and docs; it does not duplicate Stratum’s wiki — it **links** to it.

---

## 2. Relation to Stratum (links)

| Topic | Location in Stratum |
|--------|---------------------|
| ICS definition & proposal flow | `wiki/teams-proposals.md` |
| Endpoint tables (teams, proposals, notifications, …) | `wiki/api-reference.md` |
| Confidence / decay | `wiki/confidence-decay.md` |
| Semantic conflicts | `wiki/semantic-conflicts.md` |
| Vault layout on disk | `wiki/vault-layout.md` |
| Survey: shipped APIs vs older blueprint docs | `docs/superpowers/specs/2026-05-05-ics-cli-tool-repository-documentation.md` |
| Vault HTTP implementation | `backend/src/stratum/api/vaults.py` |
| Desktop sidecar (not ICS-specific) | `backend/stratum_entry.py` |

**Blueprint vs shipped:** Older backup docs describe `POST /api/vaults/{slug}/sync/diff` and legacy third-party CLI packaging. Stratum today exposes **`POST /api/vaults/{slug}/pull`** with `confirm: false` (preview diff) / `confirm: true` (apply). Milestone **C** must track **actual** routes, not the old names. This repo builds the **`ics`** binary from **Rust** (Cargo crate name **`ics-cli`**; publish to **crates.io** under that name unless renamed).

---

## 3. Product model

### 3.1 Local-first (milestone B)

- Working tree: user-edited `.md` files under a configured root.
- Local **snapshots** (commits): **hybrid storage** (decision **2026-05-05**):
  - **Content-addressed blob files** on disk for immutable bytes (per-file blobs and serialized **tree** snapshots), under `.ics/objects/…` (Git-like layout: hash prefix paths).
  - **SQLite** (e.g. `.ics/store.db`) for the **commit DAG**, **refs** (`HEAD`, branch tips), and query-friendly metadata.
  - B0–B2 **local** history does not require Stratum.
- **Online bridge:** auth → create/update/publish notes → build **proposal** payloads referencing **server note IDs**.

#### 3.1.1 `.ics/` directory layout (normative for B0)

| Path | Role |
|------|------|
| `.ics/store.db` | SQLite: commits table, refs table (and optional meta/version pragma). |
| `.ics/objects/blobs/{ab}/{rest}` | Raw bytes for each **blob** object (typically one tracked file version); `ab` = first two hex chars of SHA-256 of raw bytes. |
| `.ics/objects/trees/{ab}/{rest}` | UTF-8 JSON tree manifest for each **tree** object; `rest` from SHA-256 of canonical JSON (sorted keys, stable separators). |

Refs `HEAD` and branch names (e.g. `refs/heads/main`) map symbolic names to **commit** hashes stored in SQLite; each commit row references a **tree** object id for the snapshot.

### 3.2 Identity bridge (required for B1+)

Stratum identifies notes by `(owner_user_id, note_id)` in proposal payloads. The CLI must maintain a **deterministic mapping**:

- **Preferred:** YAML frontmatter on each file, e.g. `stratum_note_id`, `stratum_owner_id` (or a single composite), synced on first push and on pull refresh; or
- **Fallback:** a local index file mapping `relative_path → { owner_user_id, note_id }`.

Without this, `POST /api/proposals` cannot reliably attach local files to ICS.

### 3.3 Vault alignment (milestone C)

When Stratum’s vault APIs and semantics are stable enough for this tool:

- Add commands that mirror **fork / clone / file tree / pull** workflows documented in Stratum, reusing server-side diff/apply behavior where it matches team needs.
- **C does not replace B’s local commit model** — it is an **additional** sync path for deployments that standardize on Stratum vaults on disk.

---

## 4. Milestones: B0 → B2 → C

| Phase | Feels like | Stratum / network | What the CLI does |
|-------|------------|-------------------|-------------------|
| **B0** | Pure local git for markdown | **None** | `init`, `commit`, `log`, `diff`, `status`, optional `revert` on the local tree only. |
| **B1** | B0 + “remote is Stratum” | Auth + notes APIs + publish + proposals | Login; create/update note bodies from files; **publish** team-scoped notes per wiki; map IDs in frontmatter/index; `proposal create` / `proposal submit` using `POST /api/proposals` (rationale ≥ 50 chars, etc.). |
| **B2** | B1 + pull server truth into tree | B1 + list/read note content APIs | Refresh local files from server; resolve conflicts policy (last-write-wins vs manual — **spec per command**); keep mapping consistent. |
| **C** | B2 + vault-shaped sync | Mature `/api/vaults/...` usage | Fork/clone/tree; **`POST /api/vaults/{slug}/pull`** preview/confirm; align paths with Stratum vault roots; reduce custom sync where vault pull + proposals already cover merges. |

**Sequencing (agreed):** implement **B** (local workspace + ICS over HTTP) first; add **C** when the Stratum server-side vault story is “complete” enough for this tool to depend on it without constant breakage.

---

## 5. Implementation language & suggested libraries (non-binding)

- **Language:** **Rust** (edition **2021**). Primary artifact: **`ics`** binary (see `[[bin]]` in workspace `Cargo.toml`).
- **CLI:** **clap** (derive).
- **SQLite:** **rusqlite** (optionally `bundled` feature for portable builds).
- **HTTP (B1+):** **reqwest** with **tokio** async runtime (or blocking client only if you deliberately avoid async — prefer async + `tokio::main` for consistency with ecosystem).
- **JSON:** **serde** + **serde_json** (trees, commits, API bodies).
- **YAML frontmatter (B1+):** **serde_yaml**.
- **Crypto:** **sha2** + **hex** (object hashes).
- **Errors:** **anyhow** at CLI boundary; **thiserror** for library error types.
- **Config:** `STRATUM_BASE_URL`; token file under `~/.config/ics-cli/` with mode **0600** (use `std::fs::OpenOptions` + `libc`/`nix` chmod or write then `std::fs::set_permissions` where supported).

**Distribution:** **crates.io** as **`ics-cli`** (package) installing binary **`ics`**; optional release artifacts via **GitHub Releases** (static musl builds later).

---

## 6. Error handling & UX

- Surface Stratum **401 / 403 / 422** with actionable text (e.g. private note in proposal, rationale length, role restrictions).
- Display **`conflict_hints`** from proposal responses read-only (Stratum computes embeddings server-side).

---

## 7. Out of scope (v0 of this spec)

- ActivityPub / `stratum://` federation (Stratum roadmap).
- Replacing server-side embedding or conflict detection.
- Full reimplementation of Stratum’s per-user SQLite vault inside the CLI.

---

## 8. Self-review

- **Placeholders:** None; milestone table is explicit.
- **Consistency:** B then C; C uses real vault routes, not blueprint-only names.
- **Scope:** Single CLI repo + Stratum as backend; federation excluded.
- **Ambiguity:** B0 local store is fixed (**hybrid**: blobs on disk + SQLite for graph/refs). Conflict resolution policy for **B2** remains in the B2 implementation plan (explicit subcommands or flags). Language for implementation is **Rust** (§5).

---

## 9. Implementation plans

Per-milestone plans (executable task lists):

| Milestone | Document |
|-----------|----------|
| **B0** — local store + `ics init/commit/log/diff/status` | [`2026-05-05-ics-cli-b0.md`](../plans/2026-05-05-ics-cli-b0.md) |
| **B1** — Stratum auth, notes, proposals | [`2026-05-05-ics-cli-b1.md`](../plans/2026-05-05-ics-cli-b1.md) |
| **B2** — pull server state, conflicts | [`2026-05-05-ics-cli-b2.md`](../plans/2026-05-05-ics-cli-b2.md) |
| **C** — vault pull / fork / clone alignment | [`2026-05-05-ics-cli-c.md`](../plans/2026-05-05-ics-cli-c.md) |
