# Pull (B2)

## `ics pull [--all-tracked | PATH...] --policy POLICY [--force]`

Fetches note bodies from Stratum and writes them into the worktree.

### Requirements

- Valid credentials (**`ics login`**).
- **Index entry**: each pulled path must exist in **`.ics/stratum-index.json`** (normally written by **`ics push`**). Frontmatter IDs alone are **not** consulted by **`pull`** today — ensure you’ve pushed (or manually maintain the index consistently).

### `--all-tracked`

Walks **every tracked `*.md`** under the repo:

- Pulls paths that **have** an index entry.
- **Warns** and **skips** tracked files that lack index rows (so you know sync coverage is incomplete).

Alternatively pass explicit **`PATH`** arguments (resolved under the repo).

### `--policy`

| Value | Behavior |
|-------|-----------|
| **`take-theirs`** | Overwrites local file bytes with the server **`body`** string. |
| **`take-mine`** | If the local file **exists** and **differs** from the server body, **abort** with a conflict error. If the file **does not exist**, creates it from the server. If bytes already match, no-op. |

### Server response expectations

The client requires the note JSON to contain a **`body` field that is a JSON string**. Non-string or missing bodies are **errors** (no silent empty overwrite).

### Dirty working tree guard

If your **tracked markdown hashes** differ from **HEAD** (you have uncommitted local edits), **`pull` exits with code `2`** unless you pass **`--force`**.

This mirrors “don’t pull over unknown local state” — commit or **`checkout`** first, or override knowingly.

### Example

```bash
ics pull --all-tracked --policy take-theirs --force
ics pull notes/a.md notes/b.md --policy take-mine
```
