# Proposals (B1)

## `ics proposal submit --team-id ID --note-ids IDS --rationale TEXT`

Submits **`POST /api/proposals`** with a JSON body:

- **`team_id`**: numeric team identifier.
- **`note_ids`**: comma-separated list, e.g. **`--note-ids 1,2,3`**.
- **`rationale`**: human-readable rationale.

### Validation

- **`rationale`** must be at least **50 characters** before the request is sent (client-side guard aligned with common Stratum constraints).

### Authentication

Requires stored credentials from **`ics login`**.

### Errors

HTTP **`401` / `403` / `422`** surface as failures with parsed JSON hints when available (including **`conflict_hints`** text in the error detail path).

Example:

```bash
ics proposal submit \
  --team-id 1 \
  --note-ids 10,11 \
  --rationale 'Align note titles with ICS proposal scope per team policy guidelines.'
```
