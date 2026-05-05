# ics-cli usage wiki

End-user documentation for the **`ics`** command-line tool from the **`ics-cli`** Cargo package in this repository. Stratum is the network backend; this wiki focuses on **local workflows**, **configuration**, and **how commands map to HTTP** (see pinned routes in the source).

## Contents

| Article | What it covers |
|--------|----------------|
| [Overview](01-overview.md) | What `ics` is, milestones (B0–C), safety notes |
| [Installation](02-installation.md) | Building from source, running tests, binary name |
| [Local repository](03-local-workflow.md) | `init`, `status`, `commit`, `log`, `diff`, `checkout` |
| [Stratum: login & push](04-stratum-login-and-push.md) | Auth, credentials file, frontmatter & index, `push` |
| [Pull](05-pull.md) | `pull`, policies, dirty-tree guard, `--all-tracked` semantics |
| [Proposals](06-proposals.md) | `proposal submit`, rationale length, errors |
| [Vault](07-vault.md) | `vault pull`, preview vs apply, `--yes`, slug rules |
| [Configuration](08-configuration.md) | Environment variables, paths, permissions |
| [Storage layout](09-storage-layout.md) | `.ics/` on disk, blobs, trees, SQLite |
| [Troubleshooting](10-troubleshooting.md) | Common failures and exit codes |

## Quick start (local only)

```bash
cd /path/to/notes
ics init
echo '# Hello' > note.md
ics commit -m 'first snapshot'
ics log
```

## Related design docs

- Repository design spec: [`docs/superpowers/specs/2026-05-06-ics-cli-design.md`](../superpowers/specs/2026-05-06-ics-cli-design.md)
- Stratum ICS wiki (upstream): [Teams & Proposals](https://github.com/Stratum-ICS/Stratum/blob/master/wiki/teams-proposals.md)
