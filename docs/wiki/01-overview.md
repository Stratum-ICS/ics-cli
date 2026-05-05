# Overview

## What is `ics`?

`ics` is a CLI for working with **Markdown notes** in a directory tree while keeping **local, git-like history** (snapshots/commits). When you connect to **[Stratum](https://github.com/Stratum-ICS/Stratum)**, the same tool can **push** note bodies, **pull** server versions, **submit proposals**, and run **vault pull** flows — using routes pinned from an OpenAPI-derived fixture in this repo.

## Milestones (what is implemented)

| Phase | Scope | Network |
|-------|--------|---------|
| **B0** | Hybrid `.ics/` store: blobs + SQLite; `init`, `status`, `commit`, `log`, `diff`, `checkout` | None |
| **B1** | `login`, `push`, YAML frontmatter / `.ics/stratum-index.json`, `proposal submit` | Stratum HTTP |
| **B2** | `pull` with explicit `--policy` (`take-mine` \| `take-theirs`), dirty-tree guard | Stratum HTTP |
| **C** | `vault pull` preview + optional apply (`confirm` false/true) | Stratum HTTP |

Route paths are defined in `src/stratum/routes.rs` and tied to `tests/fixtures/stratum-openapi.json` (SHA in file header). **Replace the fixture** when your Stratum revision’s OpenAPI differs.

## Mental model

- **Working tree**: your editable `*.md` files (anything under the repo root except `.ics/`).
- **HEAD / main**: refs stored in SQLite pointing at the latest commit (linear history in B0).
- **Stratum mapping**: after `push`, note and owner IDs are written into **YAML frontmatter** and mirrored in **`.ics/stratum-index.json`** for paths without frontmatter fields.

## Safety and destructive commands

- **`ics checkout`** overwrites files on disk from the **HEAD** tree. You must pass explicit paths **or** `--all`.
- **`ics pull --policy take-theirs`** overwrites local files with server bodies when applied.
- **`ics vault pull`** can apply server-side changes; **`--yes`** skips confirmation and is intended for **non-interactive** use only when you accept risk.

## Path rules

CLI paths must stay **inside the repository**. Tree keys stored in objects/SQLite cannot contain `..`, absolute paths, or backslashes. Symlinked `*.md` files are **skipped** when scanning the worktree.
