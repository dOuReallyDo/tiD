# CLAUDE.md — tiD Project Guide

> Instructions for Claude Code when working on this repository.

## Project Overview

tiD is a Rust rewrite of tiC (Python/Flask), WindTre's CVM smartphone pricing tool.
Single static `.exe`, zero dependencies, zero install, zero admin rights on Windows.

## Tech Stack

- **Rust** (edition 2024) — Axum 0.8 web framework, Tokio async runtime
- **calamine** 0.26 — Excel reading (xlsb/xlsx)
- **rust_xlsxwriter** 0.82 — Excel writing
- **clap** 4 — CLI subcommands (`serve`, `compliance`, `validate`)
- **Vue 3** frontend — pre-built static files in `frontend/dist/`

## Build Commands

```bash
cargo build                      # Debug
cargo build --release            # Release (LTO + strip, ~4MB)
cargo test                       # 18 unit tests
cargo run -- serve               # Start server on port 5002
cargo run -- compliance --tolerance 0.01  # KPI compliance check
```

## Windows Build

Only via GitHub Actions CI (`build-release.yml`). Cross-compile from macOS is broken (missing MSVC headers).
Push a `v*` tag to trigger the Windows release build.

## Key Paths

- `src/paths.rs` — All directory resolution. **IMPORTANT**: `base_dir()` must return the exe's parent directory, NOT `current_dir()`. This is critical for Windows portability.
- `src/api/routes.rs` — All 17 API routes
- `src/engine/economics.rs` — Core financial math (NPV, payback, ARPU, etc.)
- `src/engine/data.rs` — Excel ingestion
- `frontend/dist/` — Pre-built Vue SPA (from tiC repo, not rebuilt here)

## Data Files

`data/sources/` contains 4 factory Excel files (~35MB total). These are git-ignored but **must be present** for the app to run. They need to be included in the distribution ZIP.

## Architecture Constraints

- All data is in-memory (no database) — same as tiC
- API must match tiC exactly for frontend compatibility
- Frontend is NOT rebuilt in this repo — copy from tiC's `dist/` if updating
- Path resolution must work relative to exe location, NOT CWD (Windows users may launch from shortcuts)

## Testing

```bash
cargo test                              # All tests
cargo test economics                    # Economics engine tests only
cargo run -- compliance --tolerance 0.01 # Compare KPIs vs tiC baseline
```

## Common Pitfalls

1. **Don't use `current_dir()` for path resolution** — always resolve from exe directory
2. **Don't modify `frontend/dist/` source code** — it's a build artifact from tiC
3. **Don't try local cross-compile for Windows** — use CI instead
4. **`data/sources/` is git-ignored** — remember to copy Excel files into the release distribution
5. **The `zip` crate dependency** brings in C libs (bzip2, lzma, zstd) — this complicates cross-compilation but is handled by CI