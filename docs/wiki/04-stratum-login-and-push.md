# Stratum: login & push (B1)

These commands require a reachable Stratum base URL and compatible API routes (see `src/stratum/routes.rs`).

## `ics login --username USER [--password PASS]`

Calls the pinned **`POST /api/auth/login`** route with JSON credentials.

- **Password**: pass **`--password`** or set **`STRATUM_PASSWORD`** in the environment (avoid shell history when possible).
- On success, stores **`credentials.json`** (see [Configuration](08-configuration.md)).
- The client accepts either **`access_token`** or **`token`** in the JSON response as the bearer secret.

## `ics push [PATH...]`

Creates or updates notes from Markdown files.

- With **no paths**, pushes **all** tracked `*.md` files.
- Each file is read as UTF-8 text. If YAML frontmatter exists between **`---`** fences, it is parsed; the **body** sent to Stratum is the markdown **after** the closing fence.

### Create vs update

- If **`stratum_note_id`** and **`stratum_owner_id`** appear in frontmatter **or** `.ics/stratum-index.json` has an entry for that relative path, `ics` uses **`PUT /api/notes/{note_id}`**.
- Otherwise it **`POST /api/notes`**.

### After a successful push

- Frontmatter is rewritten/merged to include **`stratum_note_id`** and **`stratum_owner_id`** from the response (IDs default from JSON fields; **`owner_user_id`** is read when present).
- **`.ics/stratum-index.json`** is updated for every pushed path so pulls can resolve IDs without frontmatter.

### Typical workflow

```bash
export STRATUM_BASE_URL=http://127.0.0.1:8000
ics login --username alice --password "$STRATUM_PASSWORD"
ics push notes/hello.md
ics push    # all tracked files
```

### Errors

- **`run ics login first`** if credentials are missing.
- HTTP failures surface as errors; **`422`** responses attempt to include **`detail`** / **`conflict_hints`** text when JSON bodies provide them.
