# tiD — Development Plan

## Phases

### Phase 1: Core Engine ✅ Target: v0.1
**Goal**: Economics engine + data loading + CLI compliance — prove the math matches tiC.

- [x] Create repo + Cargo project
- [ ] Set up Cargo.toml with dependencies (calamine, serde, clap, tracing)
- [ ] Port `economics_engine.py` → `src/engine/economics.rs`
  - CashflowShape struct
  - NPV calculation
  - Payback P&L and Cash
  - Bad debt (montante-based)
  - Financing cost (declining-balance)
  - Churn NPV incremental
  - Net ARPU (cluster-based)
- [ ] Port `data_manager.py` → `src/engine/data.rs`
  - MODE_COLUMN_MAP, RULE_COLUMN_MAP, ASSUMPTION_CELLS
  - Excel ingestion via calamine
  - Assumption resolution
- [ ] Port `churn_engine.py` → `src/engine/churn.rs`
- [ ] Port `paths.py` → `src/paths.rs`
- [ ] CLI compliance runner → `src/bin/compliance.rs`
- [ ] Unit tests: cashflow 24/30/36, NPV, payback, bad_debt, financing_cost
- [ ] **Validation**: compliance_runner output matches tiC ≥99.9%

### Phase 2: HTTP API — Target: v0.2
**Goal**: Full REST API compatible with tiC frontend.

- [ ] Axum server setup (port 5002)
- [ ] CORS middleware
- [ ] PricingEngine orchestrator (in-memory state)
- [ ] All API routes (see ARCHITECTURE.md API Surface)
- [ ] Request tracing (X-Request-Id)
- [ ] Static file serving for frontend
- [ ] Integration tests for each endpoint

### Phase 3: Frontend + Upload — Target: v0.3
**Goal**: Working UI in browser with file upload.

- [ ] Copy tiC Vue frontend build to frontend/dist/
- [ ] File upload endpoint (multipart)
- [ ] File save to data/inputs/ with _UPLOADED suffix
- [ ] Data reload after upload
- [ ] Churn curve table display + edit
- [ ] Batch edit UI support
- [ ] Manual UAT against tiC screenshots

### Phase 4: Export + Feature Parity — Target: v0.4
**Goal**: Full feature parity with tiC.

- [ ] xlsx export (rust_xlsxwriter)
- [ ] Formula string generation (formula_engine)
- [ ] Workbook compare (diff tool)
- [ ] Version snapshot/approve
- [ ] Full-package ZIP export
- [ ] Parametric optimizer (if needed)

### Phase 5: Windows Packaging + Release — Target: v1.0
**Goal**: Production-ready zip for Windows deployment.

- [ ] Cross-compile for x86_64-pc-windows-msvc
- [ ] Static CRT linkage
- [ ] Build release script
- [ ] Package: exe + frontend/dist/ + data/ template + START_tiD.bat
- [ ] Smoke test on Windows (or via CI)
- [ ] README with user instructions
- [ ] GitHub release with zip artifact

## Progress Log

| Date | Phase | What |
|---|---|---|
| 2025-04-25 | 1 | Repo created, Architecture + Plan written, Rust project initialized |