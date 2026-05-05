# Local repository (B0)

Commands in this section work **offline**. They discover a repo by walking parents from the current directory until a **`.ics/`** directory exists.

## `ics init [PATH]`

Creates a new repository:

- Default **`PATH`** is `.` (current directory).
- Creates **`.ics/`** with empty object directories and **`store.db`** (SQLite schema version **1**).

Errors:

- **`already an ics repository`** if `.ics` already exists.

Example:

```bash
ics init
ics init ~/notes/myproject
```

## `ics status`

Compares tracked **`*.md`** files (recursive, excluding `.ics/`) against the **HEAD** snapshot.

- If there are **no commits**, every tracked file is treated as **new** relative to an empty tree.
- Output lines look like **`A path`**, **`M path`**, **`D path`** (added / modified / deleted vs HEAD content hashes).

## `ics commit -m MESSAGE [--author NAME]`

Records a new commit:

- Snapshots all tracked **`*.md`** files.
- Updates refs **`HEAD`** and **`main`**.
- Prints the **commit id** (hex SHA-256 of canonical JSON; see design spec).

Options:

- **`--author`**: defaults to `$USER`, then `$USERNAME`, then **`unknown`**.

Errors:

- **`nothing to commit, working tree clean`** if the tree matches **HEAD** exactly.

## `ics log`

Prints commits starting from **HEAD**, walking **first parent only** (linear history). Shows Unix **`created_at`** integer today (not formatted as a calendar string).

## `ics diff [PATH...]`

Unified diff between **HEAD** and the working tree.

- With **no paths**, diffs **all** paths that differ (including “all new” when there are no commits — consistent with **`status`**).
- With paths, resolves each path under the repo and diffs matching tree keys.

## `ics checkout [--all] [PATH...]`

Restores file contents from the **HEAD** tree (**destructive**).

- You must specify **one or more `PATH`** arguments **or** pass **`--all`** to restore every path in **HEAD**.
- Creates parent directories as needed.

This is **not** `git revert`; it overwrites your working files with the last committed snapshot.

## Tips

- Only **Markdown** (`*.md`) participates in tracking.
- Hidden directories other than `.ics` are still walked; only `.ics` is excluded.
