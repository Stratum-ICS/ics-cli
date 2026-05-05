# Troubleshooting

## `not an ics repository`

Run **`ics init`** in the directory (or a parent), or **`cd`** into a folder that contains **`.ics/`**.

## `checkout: specify PATH arguments or pass --all`

**`checkout`** requires explicit paths or **`--all`** to avoid accidental full-tree restore.

## `nothing to commit, working tree clean`

There are no changes to tracked **`*.md`** files relative to **HEAD**.

## `pull` exits with code **`2`**

Your working tree **does not match HEAD** (uncommitted edits). Commit, **`checkout`**, or pass **`--force`** if you accept overwriting local-only state **before** pulling server content.

## `pull: specify PATH arguments or --all-tracked`

You must choose explicit paths or **`--all-tracked`**.

## `warning: … lack Stratum index entries`

Tracked markdown exists but was never **`push`**’d (or index manually cleared). Run **`ics push`** for those paths first.

## `vault pull: non-interactive mode requires --yes`

stdin is not a terminal (CI/job). Pass **`--yes`** only if you intend to apply without a prompt.

## `vault preview response must be a JSON object to apply`

The preview endpoint returned a non-object JSON value. The CLI refuses to send a misleading apply body; inspect server responses or adjust Stratum version/fixture.

## HTTP / auth errors

- **`401` / `403`**: renew session via **`ics login`**.
- **`422`**: read emitted **`detail`** / **`conflict_hints`** text; adjust payload (e.g. proposal rationale length).

## Route mismatch / 404 against Stratum

Regenerate **`tests/fixtures/stratum-openapi.json`** from your Stratum instance, update **`src/stratum/routes.rs`** constants and SHA comment, rebuild.

## Tests failing locally

Run **`cargo test`**. Wiremock tests bind ephemeral ports; sandboxed environments without networking loopback may break — run on a normal Linux/macOS host.
