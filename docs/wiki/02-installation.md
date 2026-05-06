# Installation

## Prebuilt binaries (GitHub Releases)

1. Open **[Releases](https://github.com/Stratum-ICS/ics-cli/releases)** and pick the latest version.
2. Download the archive for your platform:
   - **Linux x86_64:** `ics-*-x86_64-unknown-linux-gnu.tar.gz`
   - **macOS Intel:** `ics-*-x86_64-apple-darwin.tar.gz`
   - **macOS Apple Silicon:** `ics-*-aarch64-apple-darwin.tar.gz`
   - **Windows x86_64:** `ics-*-x86_64-pc-windows-msvc.zip`
3. Extract the `ics` (or `ics.exe`) binary and add it to your `PATH`, or point tools (e.g. Obsidian ICS plugin) at the full path.

Artifacts are produced when a maintainer **publishes** a GitHub Release (see repo workflow `release.yml`).

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
