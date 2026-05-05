# Configuration

## Environment variables

| Variable | Purpose | Default |
|----------|---------|---------|
| **`STRATUM_BASE_URL`** | Stratum HTTP origin | `http://127.0.0.1:8000` |
| **`STRATUM_PASSWORD`** | Password for **`ics login`** if **`--password`** omitted | unset |
| **`ICS_CONFIG_HOME`** | Override directory for **`credentials.json`** (tests / isolation) | XDG config dir via `directories` |

Trailing slashes on **`STRATUM_BASE_URL`** are stripped.

## Credentials file

Default location:

- **`$XDG_CONFIG_HOME/ics-cli/credentials.json`** (typically **`~/.config/ics-cli/credentials.json`** on Linux).

Contents (JSON):

```json
{
  "access_token": "<bearer>",
  "base_url": "http://127.0.0.1:8000"
}
```

### Permissions

On Unix, writes use a temp file in the same directory then **`rename`**, with mode **`0600`** on the temp file before replace.

### Security notes

- Do **not** commit **`credentials.json`**.
- Tokens must **not** appear in logs; avoid **`--password`** in shell history — prefer env vars or secret managers.

## Identity bridge

Two mechanisms (see design §3.2):

1. **YAML frontmatter** keys **`stratum_note_id`** and **`stratum_owner_id`** on each note.
2. Fallback **`.ics/stratum-index.json`**: map **`relative/path.md`** → **`{ owner_user_id, note_id }`**.

After **`push`**, both are kept in sync for pushed paths.
