# tiD — Development Plan

## Phases

### Phase 1: Core Engine ✅ (v0.1)
**Goal**: Economics engine + data loading + CLI compliance — prove the math matches tiC.

- [x] Create repo + Cargo project
- [x] Set up Cargo.toml with dependencies (calamine, serde, clap, tracing, axum, tokio, rust_xlsxwriter)
- [x] Port `economics_engine.py` → `src/economics.rs`
  - CashflowShape struct
  - NPV calculation
  - Payback P&L and Cash
  - Bad debt (montante-based)
  - Financing cost (declining-balance)
  - Churn NPV incremental
  - Net ARPU (cluster-based)
- [x] Port `data_manager.py` → `src/data_manager.rs`
  - MODE_COLUMN_MAP, RULE_COLUMN_MAP, ASSUMPTION_CELLS
  - Excel ingestion via calamine
  - Assumption resolution
- [x] Port `churn_engine.py` → churn tables in `src/economics.rs`
- [x] Port `cashflow.rs` — 24/30/36 month structures with `ultima_rata`
- [x] Port `compliance.rs` — KPI comparison vs baseline
- [x] Port `formula_engine.rs` — formula string generation
- [x] Port `workbook_editor.rs` — xlsx export/write
- [x] CLI compliance runner (`cargo run -- compliance`)
- [x] Unit tests: 18/18 passed (cashflow, NPV, payback, bad_debt, financing_cost, ARPU)
- [x] **Validation**: compliance_runner output matches tiC ≥99.9%

### Phase 2: HTTP API ✅ (v0.2)
**Goal**: Full REST API compatible with tiC frontend.

- [x] Axum server setup (port 5002)
- [x] CORS middleware
- [x] PricingEngine orchestrator (in-memory state)
- [x] All API routes — 18 endpoints (products, kpi, compliance, upload, export, etc.)
- [x] Request tracing (X-Request-Id)
- [x] Integration tests for each endpoint

### Phase 3: Frontend + Upload ✅ (v0.3)
**Goal**: Working UI in browser with file upload.

- [x] Copy tiC Vue frontend build to frontend/dist/
- [x] Static file serving via Axum ServeDir (SPA fallback)
- [x] File upload endpoint (multipart)
- [x] File save to data/sources/ with _UPLOADED suffix
- [x] Data reload after upload
- [x] Churn curve table display + edit
- [x] Batch edit UI support
- [x] Windows CI via GitHub Actions (build-release.yml)
- [x] START_tiD.bat startup script

### Phase 4: Export + Feature Parity ✅ (v0.4)
**Goal**: Full feature parity with tiC.

- [x] xlsx export with live formulas (formula_engine → cells with Excel formulas)
- [x] Formula string generation in output cells (formula_engine integration)
- [x] Workbook compare (diff tool) — compare two Excel files cell-by-cell
- [x] Version snapshot/approve workflow — save & label versions
- [x] Full-package ZIP export (exe + frontend + data template)
- [ ] Parametric optimizer (deferred to v0.5)

### Phase 5: Windows Packaging + Release — ✅ v1.0
**Goal**: Production-ready zip for Windows deployment.

- [x] Cross-compile for x86_64-pc-windows-msvc (via CI)
- [x] Static CRT linkage (`-C target-feature=+crt-static`)
- [x] Build release script (GitHub Actions)
- [x] Package: exe + frontend/dist/ + data/sources/ + START_tiD.bat
- [x] data/sources/ Excel files bundled in ZIP (CI + build_release.sh)
- [x] BUG FIX: base_dir() usa current_exe() — portabile su Windows da qualsiasi CWD
- [x] BUG FIX: frontend title corretto ("tiD" invece di "tiC")
- [x] README user-facing v1.0 (deploy Windows, istruzioni Excel, sezione developer)
- [ ] Smoke test on actual Windows machine (da fare su VM o hardware fisico)
- [x] GitHub release with zip artifact

## Progress Log

| Date | Phase | What |
|---|---|---|
| 2026-04-25 | 1 | Repo created, Architecture + Plan written, Rust project initialized |
| 2026-04-25 | 1 | Core engine ported: economics, data_manager, cashflow, compliance, formula_engine, workbook_editor |
| 2026-04-25 | 1 | 18/18 unit tests passing, cargo build clean |
| 2026-04-26 | 2 | Axum server, 18 API endpoints, CORS, tracing |
| 2026-04-27 | 3 | Vue frontend dist copied, SPA serving, upload endpoint, START_tiD.bat |
| 2026-04-27 | 3 | CI workflow (build-release.yml), Windows build passing, v0.3.0 tagged |
| 2026-04-27 | 3 | CI release failed: 403 permissions — fix: add `permissions: contents: write` |
| 2026-05-15 | 5 | BUG FIX: paths.rs base_dir() → current_exe(); frontend title tiC→tiD; data/sources in CI+script; README v1.0; tag v1.0.0 |