# tiD — Architecture

> **tiC reimagined in Rust** — CVM Pricing Cockpit as a single self-contained Windows executable.

## Overview

tiD is a complete Rust rewrite of [tiC](https://github.com/dOuReallyDo/tiC), WindTre's CVM smartphone pricing tool. It runs as a single `.exe` with zero dependencies — no Python, no Node.js, no admin rights. The user unzips, drops Excel files in `data/`, double-clicks, and opens a browser.

## System Requirements

| Requirement | Constraint |
|---|---|
| OS | Windows 10/11 (x86_64) |
| RAM | ≤ 16 GB (OS + other apps share this) |
| Admin rights | None |
| Installation | None — unzip and run |
| Dependencies | Zero — fully static binary |

## Technology Stack

| Component | Technology | Rationale |
|---|---|---|
| Language | **Rust** (edition 2024) | Zero-cost abstractions, no runtime, single-binary output |
| HTTP Server | **Axum** | Tokio-based, type-safe routing, ~2MB overhead |
| Async Runtime | **Tokio** | Industry standard, efficient with limited RAM |
| Excel Reading | **calamine** | Same library as tiC (Rust-native, handles external refs) |
| Excel Writing | **rust_xlsxwriter** | Pure Rust, no COM/OLE, fast xlsx generation |
| Serialization | **serde + serde_json** | JSON API responses, deserialization of configs |
| CLI | **clap** | Compliance runner subcommand, arg parsing |
| Logging | **tracing** | Structured logging, zero overhead when disabled |
| Frontend | **Vue 3 build** (pre-compiled, served as static files) | Same UX as tiC |

## Architecture Diagram

```
┌──────────────────────────────────────────────────┐
│                  tiD.exe (~12MB)                  │
├──────────────────────────────────────────────────┤
│  Axum HTTP Server (port 5002)                    │
│  ├── /api/*          → JSON API routes           │
│  ├── /upload         → File upload endpoint       │
│  └── /*              → Static frontend (Vue)      │
├──────────────────────────────────────────────────┤
│  Core Engine                                     │
│  ├── PricingEngine   → Orchestration singleton    │
│  ├── EconomicsEngine → NPV, Payback, ARPU, etc.  │
│  ├── DataManager     → Excel ingestion (calamine) │
│  ├── ChurnEngine     → Curve interpolation        │
│  └── FormulaEngine   → Excel formula string gen   │
├──────────────────────────────────────────────────┤
│  Export Layer                                     │
│  ├── WorkbookWriter  → xlsx generation (xlsxwriter)│
│  └── WorkbookCompare → Diff source vs exported    │
├──────────────────────────────────────────────────┤
│  CLI Subcommands (via clap)                       │
│  ├── tiD serve       → Start HTTP server          │
│  ├── tiD compliance  → KPI score vs baseline     │
│  └── tiD validate     → Data integrity check      │
└──────────────────────────────────────────────────┘
         │
    data/ directory (user-managed)
    ├── sources/  → Factory Excel files
    ├── inputs/   → User-uploaded overrides
    └── exports/  → Generated xlsx output
```

## Data Flow (identical to tiC)

```
Excel files → DataManager (calamine) → PricingEngine (in-memory)
                                              │
                              EconomicsEngine.compute_kpis()
                                              │
                              Axum API → Vue frontend (browser)
                                              │
                              PricingEngine.export_*() → xlsx via WorkbookWriter
```

## Module Map (tiC → tiD)

| tiC (Python) | tiD (Rust) | Notes |
|---|---|---|
| `app.py` | `src/api/` | Axum router, middleware, error handling |
| `pricing_engine.py` | `src/engine/pricing.rs` | Orchestration, product state, versioning |
| `economics_engine.py` | `src/engine/economics.rs` | Pure financial math — direct port |
| `data_manager.py` | `src/engine/data.rs` | Excel ingestion, column maps, assumptions |
| `formula_engine.py` | `src/engine/formula.rs` | Excel formula string generation |
| `churn_engine.py` | `src/engine/churn.rs` | Churn curve interpolation |
| `workbook_editor.py` | `src/export/writer.rs` | xlsx write/patch |
| `workbook_compare.py` | `src/export/compare.rs` | Cell-level diff |
| `compliance_runner.py` | `src/bin/compliance.rs` | CLI subcommand |
| `validate_engine.py` | `src/bin/validate.rs` | CLI subcommand |
| `paths.py` | `src/paths.rs` | Path resolution (compile-time constants) |
| Vue frontend | `frontend/dist/` | Pre-built, embedded via `include_dir!` or served from disk |

## Key Design Decisions

### 1. Static Binary vs Dynamic
Cross-compile from macOS → Windows using `x86_64-pc-windows-msvc` target with `cargo build --release`. The resulting `.exe` is fully static (no MSVC runtime dependency via `RUSTFLAGS="-C target-feature=+crt-static"`).

### 2. Memory Strategy
- All products loaded in-memory (~888 products × 7 modes = ~6K KPI sets ≈ 5MB)
- No database — pure in-memory like tiC
- HashMap for O(1) product lookup
- Pre-allocated Vec for cashflows (max 37 elements)

### 3. Frontend Serving
- Option A (v1): Serve `frontend/dist/` from adjacent directory (zip contains both exe + frontend/)
- Option B (v2): Embed static files in binary via `include_dir!` crate

### 4. File Upload
- Axum `multipart` extraction for drag-and-drop file upload
- Files saved to `data/inputs/` with `_UPLOADED` suffix
- Same `resolve_input()` logic as tiC (inputs override sources)

### 5. Churn Curves
- Display as editable tables (no chart library needed)
- Data loaded from ASSUMPTIONS/CHURN sheet in ECONOMICS xlsx
- Optional: load from separate churn Excel file

## API Surface (100% tiC compatible)

All tiC API endpoints replicated for frontend compatibility:

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/health` | Health check |
| GET | `/api/products` | Product list with KPIs |
| GET | `/api/product/:id` | Single product detail |
| POST | `/api/product/:id/edit` | Edit product parameters |
| POST | `/api/product/:id/approve` | Snapshot version |
| POST | `/api/export/economics` | Export economics xlsx |
| POST | `/api/export/fasce` | Export fasce xlsx |
| POST | `/api/export/fasce_request` | Export fasce request |
| POST | `/api/export/fasce_config` | Export fasce config |
| POST | `/api/export/full-package` | Export ZIP of all |
| POST | `/api/upload` | Upload Excel files |
| GET | `/api/assumptions` | Current assumptions |
| POST | `/api/assumptions` | Update assumptions |
| GET | `/api/churn` | Churn curves |
| POST | `/api/churn` | Update churn curves |
| POST | `/api/batch-edit` | Batch edit products |
| GET | `/api/versions` | Version history |
| POST | `/api/compliance` | Run compliance check |

## Performance Targets

| Metric | tiC (Python) | tiD (Rust) | Expected Improvement |
|---|---|---|---|
| Cold start | ~3s | <200ms | 15x |
| KPI calc (888 products) | ~1.2s | <50ms | 24x |
| Export xlsx | ~2s | <300ms | 7x |
| RAM usage | ~350MB | ~30MB | 12x |
| Binary size | 80MB+ (Python embed) | ~12MB | 7x |

## Cross-Compilation Setup

From macOS development machine:
```bash
# Install Windows target
rustup target add x86_64-pc-windows-msvc

# Install linker (via Homebrew)
brew install mingw-w64

# Build
CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER=x86_64-w64-mingw32-gcc \
  cargo build --release --target x86_64-pc-windows-msvc
```

For CI: GitHub Actions with `windows-latest` runner for guaranteed compatibility.

## Directory Structure

```
tiD/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── ARCHITECTURE.md
├── PLAN.md
├── CHANGELOG.md
├── .gitignore
├── src/
│   ├── main.rs              # Entry point (CLI + server)
│   ├── lib.rs               # Re-exports
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs        # Axum router definition
│   │   ├── products.rs      # Product endpoints
│   │   ├── export.rs        # Export endpoints
│   │   ├── upload.rs        # File upload endpoints
│   │   ├── assumptions.rs   # Assumptions endpoints
│   │   ├── churn.rs         # Churn endpoints
│   │   ├── compliance.rs    # Compliance endpoint
│   │   └── error.rs         # API error types
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── pricing.rs       # PricingEngine (orchestration)
│   │   ├── economics.rs     # EconomicsEngine (KPI calc)
│   │   ├── data.rs          # DataManager (Excel ingestion)
│   │   ├── formula.rs       # FormulaEngine (Excel formula strings)
│   │   ├── churn.rs         # ChurnEngine (curve interpolation)
│   │   └── types.rs         # Shared types (CashflowShape, etc.)
│   ├── export/
│   │   ├── mod.rs
│   │   ├── writer.rs        # WorkbookWriter (xlsx output)
│   │   └── compare.rs       # WorkbookCompare (diff)
│   └── paths.rs             # Path resolution
├── tests/
│   ├── economics_test.rs    # Unit tests for KPI calculations
│   ├── cashflow_test.rs     # Cashflow structure tests
│   ├── compliance_test.rs   # Compliance runner tests
│   └── api_test.rs          # API integration tests
├── frontend/
│   └── dist/                # Pre-built Vue frontend (from tiC)
├── data/
│   ├── sources/             # Factory Excel files (git-ignored)
│   ├── inputs/              # User upload overrides (git-ignored)
│   └── exports/             # Generated exports (git-ignored)
└── scripts/
    └── build_release.sh     # Cross-compile + package script
```