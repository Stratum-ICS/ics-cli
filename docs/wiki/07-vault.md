# Vault pull (C)

## `ics vault pull SLUG [--yes]`

Uses the pinned **`POST /api/vaults/{slug}/pull`** route.

### Slug rules

For safety against URL injection, **`SLUG`** must be a **single path segment**:

- No **`/`**, **`\`**, or **`..`**.
- Example valid slug: **`my-team-notes`**.

### Flow

1. **Preview**: sends JSON with **`confirm: false`** (merged into the request body; default body is `{}`).
2. Prints the JSON response (pretty-printed).
3. **Apply**:
   - If **`--yes`**: sends **`confirm: true`** without prompting (destructive; for automation).
   - Else, if **stdin is a TTY**: prompts with **`dialoguer`**.
   - Else: **fails** — non-interactive environments **must** pass **`--yes`** explicitly.

The apply step **reuses the preview JSON value** as the base object and sets **`confirm: true`**. If the preview response is **not** a JSON **object**, the tool **refuses** to apply (prevents sending an empty `{}` apply by mistake).

### Automation warning

**`--yes`** can apply server-side changes without human review of the preview. Use only in trusted pipelines.

Example (interactive):

```bash
ics vault pull myvault
```

Example (CI):

```bash
ics vault pull myvault --yes
```
