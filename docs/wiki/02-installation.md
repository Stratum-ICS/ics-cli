# Installation

## From source (recommended during development)

Requirements:

- **Rust** toolchain (`cargo`, `rustc`), edition **2021**.
- **SQLite**: provided via `rusqlite` with the **`bundled`** feature (no system SQLite required for normal builds).

Clone the repository, then:

```bash
cd ics-cli
cargo build --release
```

The binary is **`ics`** (Cargo `[[bin]]` name). Run:

```bash
./target/release/ics --help
```

Install into your Cargo bin path (optional):

```bash
cargo install --path .
```

That installs the package **`ics-cli`** from crates.io naming when published; the executable name remains **`ics`**.

## Verify the build

```bash
cargo test
```

All integration tests should pass, including Wiremock-based HTTP stubs (no live Stratum server required).

## Smoke test (local repo)

```bash
mkdir -p /tmp/ics-demo && cd /tmp/ics-demo
/path/to/ics init
printf '%s\n' '# Demo' > note.md
/path/to/ics commit -m 'init content'
/path/to/ics log
```
