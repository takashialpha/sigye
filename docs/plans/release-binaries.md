# Plan: Pre-built release binaries (issue #37)

## Goal
Provide pre-built `sigye` binaries for multiple platforms, attached to GitHub
Releases, so users can install without the Rust toolchain.

## Key facts (verified against Cargo.lock)
- Binary name: `sigye`, workspace member at `crates/sigye` (manifest path
  `crates/sigye/Cargo.toml`). Workspace root `Cargo.toml`.
- **No C system-library build deps on Linux:**
  - `arboard` -> pure-Rust `x11rb` (no libxcb dev libs)
  - `notify-rust` -> pure-Rust `zbus` (no libdbus dev libs)
  - `ureq` -> `rustls` (no OpenSSL)
  => Cross-compilation and musl static builds are clean (no `apt-get` needed).
- Release profile already optimized (lto, strip, opt-level "s").
- Existing `publish.yml` triggers on `release: published` (crates.io publish).

## Deliverable
New workflow: `.github/workflows/release.yml`

### Triggers
- `release: { types: [published] }`  (aligns with publish.yml; uploads assets to
  the just-published release; `github.ref` = the tag)
- `workflow_dispatch` (manual; for testing builds)

### Permissions
- `contents: write` (needed to upload release assets)

### Build matrix (target -> runner)
- `x86_64-unknown-linux-gnu`   -> ubuntu-latest
- `x86_64-unknown-linux-musl`  -> ubuntu-latest  (static)
- `aarch64-unknown-linux-gnu`  -> ubuntu-latest  (cross)
- `x86_64-apple-darwin`        -> macos-13       (intel)
- `aarch64-apple-darwin`       -> macos-latest   (apple silicon)
- `x86_64-pc-windows-msvc`     -> windows-latest

`fail-fast: false` so one target failing doesn't kill the others.

### Build/upload approach
Use `taiki-e/upload-rust-binary-action@v1` per matrix entry:
- `bin: sigye`
- `target: ${{ matrix.target }}`
- `manifest-path: crates/sigye/Cargo.toml`  (workspace member)
- `archive: sigye-$target`  (tar.gz on unix, zip on windows automatically)
- `checksum: sha256`
- `locked: true`  (build with --locked since Cargo.lock is committed)
- `token: ${{ secrets.GITHUB_TOKEN }}`
- For non-native Linux targets (`aarch64-unknown-linux-gnu`,
  `x86_64-unknown-linux-musl`) the action uses `cross`/cargo automatically;
  since there are no C deps, no extra system packages are required.

The action only uploads to a release when run in a tag/release context. On
`workflow_dispatch` (no tag) it will still build (validating the matrix) but
won't have a release to upload to — that's acceptable for a manual test run.
Pin the action to its `@v1` major tag.

### README update
Add a "Pre-built binaries" subsection under Installation pointing users to the
GitHub Releases page, noting the available platforms.

## Out of scope
- Auto-creating releases from tag pushes (maintainer creates releases manually,
  which is what publish.yml already keys off of).
- Package-manager distribution (homebrew, AUR, etc.).

## Verification
- `actionlint` (or yaml parse) on the workflow file.
- Confirm matrix targets are valid rustc target triples.
- Confirm trigger does not conflict/duplicate publish.yml's job.
