# Storage layout (`.ics/`)

Normative layout for local hybrid storage:

| Path | Role |
|------|------|
| **`.ics/store.db`** | SQLite: **`commits`**, **`refs`** (`HEAD`, **`main`**), **`PRAGMA user_version = 1`**. |
| **`.ics/objects/blobs/ab/rest…`** | Raw blob bytes; **`ab`** = first two hex chars of SHA-256 of content. |
| **`.ics/objects/trees/ab/rest…`** | Canonical JSON tree manifest (sorted map path → blob hash hex). |
| **`.ics/stratum-index.json`** | Optional Stratum ID index (JSON object map). |

## Commit identity

Commit rows store:

- **`tree_id`**: hex-encoded SHA-256 of tree object.
- **`parent_commit_ids`**: JSON array text matching the canonical commit JSON **`parents`** field.

Commit id = SHA-256 hex of UTF-8 JSON object with sorted keys:

`author`, `created_at`, `message`, `parents`, `tree_id`.

## Refs

Flat names **`HEAD`** and **`main`** are updated together on **`commit`**.

## Garbage / orphans

Failed commits mid-flight could theoretically orphan objects under **`.ics/objects/`**; future tooling could GC unreachable objects. Not implemented in v0.
