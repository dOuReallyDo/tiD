# tiD — Developer Guide

> Guida tecnica completa per sviluppo futuro e troubleshooting.
> Ultimo aggiornamento: 2026-05-15 (v1.0.0)

---

## Indice

1. [Contesto e scopo](#1-contesto-e-scopo)
2. [Stack tecnologico](#2-stack-tecnologico)
3. [Struttura del repository](#3-struttura-del-repository)
4. [Architettura interna](#4-architettura-interna)
5. [Engine economico](#5-engine-economico)
6. [Ingestion dati Excel](#6-ingestion-dati-excel)
7. [API REST — Riferimento completo](#7-api-rest--riferimento-completo)
8. [Build e rilascio](#8-build-e-rilascio)
9. [Test](#9-test)
10. [Configurazione e assunzioni](#10-configurazione-e-assunzioni)
11. [Deployment Windows](#11-deployment-windows)
12. [Troubleshooting](#12-troubleshooting)
13. [Roadmap futura](#13-roadmap-futura)
14. [Decisioni architetturali](#14-decisioni-architetturali)

---

## 1. Contesto e scopo

**tiD** è una riscrittura completa in Rust di **tiC** (Python/Flask), il tool CVM Pricing Cockpit di WindTre.

tiC era funzionale ma distribuibile solo con un installer Python pesante (~80MB) e dipendeva da un runtime Flask. L'obiettivo di tiD era eliminare ogni dipendenza esterna e produrre un singolo `.exe` Windows di ~4MB che l'utente puó decomprimere e avviare senza diritti amministratore.

**Invariante fondamentale**: la logica economica di tiD deve corrispondere a tiC con un margine ≤0.01% (compliance ≥99.9%). Qualsiasi modifica all'engine va verificata con `cargo run -- compliance --tolerance 0.01`.

### Relazione con tiC

- **Repo tiC**: `https://github.com/dOuReallyDo/tiC`
- Il frontend Vue 3 è costruito dal repo tiC e copiato in `frontend/dist/`. tiD non ricostruisce il frontend.
- Le API di tiD replicano 1:1 quelle di tiC (stesso path, stessi payload JSON) per compatibilità con il frontend.
- I file Excel factory (`data/sources/`) sono gli stessi usati da tiC.

---

## 2. Stack tecnologico

| Componente | Libreria | Versione | Note |
|---|---|---|---|
| Linguaggio | Rust | edition 2024 | Binario statico, zero runtime |
| HTTP Server | Axum | 0.8 | Type-safe routing, Tokio-based |
| Async runtime | Tokio | 1.x | `features = ["full"]` |
| Excel reading | calamine | 0.26 | Supporta xlsx, xlsb, xls |
| Excel writing | rust_xlsxwriter | 0.82 | Pure Rust, genera xlsx con formule |
| Serialization | serde + serde_json | 1.x | JSON per API, derive macro |
| CLI | clap | 4.x | Subcommand: serve, compliance, validate |
| Logging | tracing | 0.1 | Structured, zero overhead |
| Utilità | anyhow, uuid, chrono, zip | — | Error handling, ID gen, timestamp, ZIP export |
| Frontend | Vue 3 (pre-built) | — | Copiato da tiC, servito come static files |

---

## 3. Struttura del repository

```
tiD/
├── Cargo.toml                   ← dipendenze e profili di build
├── Cargo.lock                   ← versioni pinned (gitignored — binario dev)
├── src/
│   ├── main.rs                  ← entry point: CLI parsing + avvio server
│   ├── lib.rs                   ← re-esporta i moduli pubblici
│   ├── paths.rs                 ← risoluzione path (CRITICO per Windows)
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs            ← Axum router: definizione di tutti i 18 endpoint
│   │   ├── products.rs          ← handler: list, get, edit, approve, batch-edit, versions
│   │   ├── export.rs            ← handler: economics, fasce, full-package, compare
│   │   ├── upload.rs            ← handler: upload file Excel via multipart
│   │   ├── assumptions.rs       ← handler: GET/POST assunzioni
│   │   ├── churn.rs             ← handler: GET/POST curve churn
│   │   ├── compliance.rs        ← handler: POST compliance check
│   │   └── error.rs             ← ApiError enum → HTTP response
│   ├── engine/
│   │   ├── mod.rs
│   │   ├── pricing.rs           ← PricingEngine: orchestrazione + SharedEngine (Arc<RwLock>)
│   │   ├── economics.rs         ← EconomicsEngine: NPV, payback, bad debt, financing cost
│   │   ├── data.rs              ← DataManager: ingestion Excel via calamine
│   │   ├── formula.rs           ← FormulaEngine: generazione stringhe formula Excel
│   │   ├── churn.rs             ← ChurnEngine: interpolazione curve di churn
│   │   ├── types.rs             ← tipi condivisi: Product, ModeState, KpiResult, ecc.
│   │   └── versioning.rs        ← snapshot/approve workflow per versioni
│   └── export/
│       ├── mod.rs
│       ├── writer.rs            ← WorkbookWriter: scrittura xlsx via rust_xlsxwriter
│       ├── compare.rs           ← WorkbookCompare: diff cell-by-cell tra xlsx
│       └── package.rs           ← ZIP export: bundling exe + frontend + data
├── tests/
│   └── economics_test.rs        ← 18 test unitari su cashflow, NPV, payback, bad debt
├── src/bin/
│   └── compliance.rs            ← binario separato: tid-compliance (non usato in prod)
├── frontend/
│   └── dist/                    ← Vue SPA pre-built (da tiC). NON modificare.
│       ├── index.html
│       └── assets/
│           ├── index-*.js       ← bundle Vue minificato
│           └── index-*.css
├── data/
│   ├── sources/                 ← file Excel factory (ora tracciati in git da v1.0)
│   │   ├── Listino_CVM_ECONOMICS.xlsx   ← prodotti + assunzioni + churn
│   │   ├── Listino_CVM_FASCE.xlsx       ← regole fasce per modo
│   │   ├── output_TI_CVM.xlsb           ← output di riferimento tiC (usato per compliance)
│   │   └── output_TI_CVM.xlsx           ← stessa cosa in formato xlsx
│   ├── inputs/                  ← override utente (gitignored)
│   └── exports/                 ← output generati (gitignored)
├── scripts/
│   ├── build_release.sh         ← build locale macOS + packaging
│   └── release_v1.sh            ← script commit + tag v1.0 (one-shot, da non riutilizzare)
├── .github/workflows/
│   └── build-release.yml        ← CI GitHub Actions: build Windows + release ZIP
├── .cargo/
│   └── config.toml              ← configurazione Cargo (target defaults)
├── START_tiD.bat                ← avvio Windows (apre browser + lancia exe)
├── START_tiD.command            ← avvio macOS (sviluppo)
├── CLAUDE.md                    ← istruzioni per Claude Code
├── HANDOFF_MVP.md               ← documento handoff v1.0 (storico)
├── PLAN.md                      ← roadmap a fasi (aggiornare a ogni release)
├── ARCHITECTURE.md              ← overview architettura (complementare a questa guida)
└── README.md                    ← istruzioni utente finale
```

---

## 4. Architettura interna

### Flusso di avvio

```
tiD.exe serve
    │
    ├─ paths::base_dir()              → exe parent directory (CRITICO)
    ├─ paths::ensure_data_dirs()      → crea data/{sources,inputs,exports,archive} se assenti
    ├─ PricingEngine::load()
    │     ├─ DataManager::load_all()  → legge i 3 file Excel in memoria
    │     └─ EconomicsEngine::calculate_mode_kpis() × (prodotti × 7 modi)
    └─ Axum::serve(create_router(engine), addr)
```

### Stato in memoria

`PricingEngine` è l'unico stato dell'applicazione. È wrappato in `Arc<RwLock<PricingEngine>>` e passato come Axum state a tutti i router:

```rust
pub type SharedEngine = Arc<RwLock<PricingEngine>>;
```

- **Write lock**: acquisito da edit, upload, batch-edit, approve
- **Read lock**: acquisito da tutti i GET

L'app non usa database. Tutto è in RAM. Al riavvio, i dati vengono ricaricati dagli Excel.

### Request routing

```
Browser → Axum
    ├─ /api/*         → handler Rust (JSON)
    └─ /* (fallback)  → ServeDir(frontend/dist/) con fallback su index.html
                        (necessario per Vue Router in history mode)
```

Il fallback SPA è fondamentale: Vue Router usa path reali (es. `/product/123`). Se Axum non restituisse `index.html` su questi path, il refresh del browser darebbe 404.

---

## 5. Engine economico

### Struttura dei dati

Ogni **prodotto** (`Product`) ha:
- Metadati: `id`, `name`, `tp` (transfer price), `full_price`, `cluster`, `profile`, `duration`
- Sette **modi** (`HashMap<String, ModeState>`): `VAR_CC`, `VAR_RID`, `FIN_COMPASS`, `FIN_FINDO`, `RELOAD_CC`, `RELOAD_COMP`, `RELOAD_FINDO`

Ogni modo ha un `ModeState` con i parametri di pricing e il risultato KPI calcolato (`KpiResult`).

### KPI calcolati

Per ogni prodotto × modo, `EconomicsEngine::calculate_mode_kpis()` produce:

| KPI | Tipo | Descrizione |
|---|---|---|
| `npv` | f64 | Net Present Value totale (€) |
| `npv_installment` | f64 | NPV delle rate |
| `npv_incremental` | f64 | NPV incrementale da churn |
| `bad_debt` | f64 | Rettifica per insolvenze |
| `financing_cost` | f64 | Costo finanziamento (solo FIN_*) |
| `pb_pl` | i32 | Payback P&L in mesi (-1 = mai) |
| `pb_cash` | i32 | Payback cash (discounted) in mesi |
| `status` | String | "PASS" o "ALERT" |
| `target_pb` | i32 | Soglia payback per il modo (12 o 16 mesi) |
| `monthly_net` | f64 | Rata mensile netta (rata_hs - sconto_mese) |

### Cashflow

Le strutture cashflow dipendono dalla durata del contratto:

- **24 mesi**: CF_0 + 23 rate mensili + ultima_rata (25 elementi)
- **30 mesi**: CF_0 + 24 rate HS + 5 rate SMART + ultima_rata (31 elementi)
- **36 mesi**: CF_0 + 35 rate mensili + ultima_rata (37 elementi)

`CF_0 = -tp + anticipo` (uscita cassa al momento 0).

### Formule chiave

```
NPV = Σ(CF_t / (1 + monthly_rate)^t)
    dove monthly_rate = (1 + annual_rate)^(1/12) - 1

Payback P&L = primo mese t dove Σ(CF_0..CF_t) ≥ 0

Bad debt = montante × bad_debt_rate
    dove montante = fascia - anticipo

Financing cost = Σ(remaining_t × spread / 12) / (1 + monthly_rate)^t
    dove spread = rate_internal - rate_customer
```

### Tassi di sconto per modo

I tassi annui sono letti dalle assunzioni Excel (fallback hardcoded):

| Modo | Chiave assunzione | Default |
|---|---|---|
| VAR_CC, RELOAD_CC | `BAD_DEBT_CC` | 8.1% |
| VAR_RID | `BAD_DEBT_RID` | 9.36% |
| FIN_COMPASS, RELOAD_COMP | `RATE_COMPASS` | 8.3% |
| FIN_FINDO, RELOAD_FINDO | `RATE_FINDO` | 8.05% |

### Target payback per modo

| Gruppo | Chiave | Default |
|---|---|---|
| VAR_*, RELOAD_* | `TARGET_PB_VAR` | 12 mesi |
| FIN_* | `TARGET_PB_FIN` | 16 mesi |

### Churn NPV incrementale

Solo se le curve churn sono disponibili nel file Excel. Calcola il valore incrementale generato dall'azione commerciale rispetto al no-action, applicando:

```
arpu_action = net_arpu - sconto_tariffa / 1.22
```

Il `ChurnEngine` interpola le probabilità di churn mese per mese e calcola l'NPV della differenza.

### Cluster e Net ARPU

Il cluster di un prodotto (`product.cluster`) determina il tasso ARPU da usare. Mappa:

| Cluster | Chiave assunzione |
|---|---|
| 1 / A | `NA_A` |
| 2 / B | `NA_B` |
| 3 / C | `NA_C` |
| 4 / D | `NA_D` |
| 5 / E | `NA_E` |
| CB | `NA_CB` |
| NT | `NA_NT` |
| *_PK | `NA_*_PK` (variante prepagato) |

---

## 6. Ingestion dati Excel

### File letti all'avvio

`DataManager::load_all()` cerca i file nel seguente ordine:

1. `data/inputs/<nome>_UPLOADED.xlsx` (override utente, caricato via UI)
2. `data/inputs/<nome>.xlsx`
3. `data/sources/<nome>.xlsx` (factory)

Questo permette di sovrascrivere i file factory senza modificarli.

### Listino_CVM_ECONOMICS.xlsx

Contiene tre fogli rilevanti:

**Sheet `LISTINO_CVM`** — prodotti e parametri per modo. Le colonne sono a posizione fissa (0-indexed):

| Modo | Colonne principali |
|---|---|
| VAR_CC | 26–33 (status, fascia, anticipo, importo_smart, sconto_tariffa, rata_hs, rata_smart, ultima_rata) |
| VAR_RID | 52–59 |
| FIN_COMPASS | 78–85 |
| FIN_FINDO | 110–117 |
| RELOAD_CC | 142–149 |
| RELOAD_COMP | 171–178 |
| RELOAD_FINDO | 200–207 |

KPI di baseline da Excel (per compliance):

| Modo | npv | pb_pl | pb_cash |
|---|---|---|---|
| VAR_CC | col 44 | col 43 | col 49 |
| VAR_RID | col 70 | col 69 | col 75 |
| FIN_COMPASS | col 96 | col 95 | col 101 |
| FIN_FINDO | col 128 | col 127 | col 133 |
| RELOAD_CC | col 160 | col 159 | col 165 |
| RELOAD_COMP | col 189 | col 188 | col 194 |
| RELOAD_FINDO | col 218 | col 217 | col 223 |

**Sheet `ASSUMPTIONS`** — valori letti per cella (es. `B5`, `D6`). Vedi mappa in `src/engine/data.rs::ASSUMPTION_CELLS`.

**Sheet `CHURN`** (opzionale) — curve churn action/no-action su 42 mesi.

### Listino_CVM_FASCE.xlsx

Regole fasce per modo — usate per determinare i parametri di pricing in base alla fascia scelta. Merge nel `ModeState.rule`.

### output_TI_CVM.xlsb / .xlsx

Output di tiC usato come baseline per il compliance check. I KPI calcolati da tiD vengono confrontati con quelli letti da questo file.

---

## 7. API REST — Riferimento completo

Tutti gli endpoint sono su `http://127.0.0.1:5002`. CORS aperto (`*`) — non adatto a produzione multi-utente.

### Health

```
GET /api/health
→ { "status": "ok" }
```

### Prodotti

```
GET /api/products
→ Array di Product con KpiResult per ogni modo

GET /api/product/{id}
→ Product singolo con KpiResult

POST /api/product/{id}/edit
Body: { "mode": "VAR_CC", "field": "fascia", "value": 299.99 }
Campi editabili: fascia, anticipo, importo_smart, rata_hs, rata_smart, sconto, sconto_tariffa
→ Product aggiornato con KPI ricalcolati

POST /api/product/{id}/approve
Body: { "label": "v2024-Q1" }
→ Salva snapshot versione

GET /api/versions
→ Lista versioni salvate

POST /api/batch-edit
Body: [{ "id": "...", "mode": "...", "field": "...", "value": 0.0 }, ...]
→ Array prodotti aggiornati
```

### Assunzioni

```
GET /api/assumptions
→ HashMap<String, Assumption> { cell, value, label }

POST /api/assumptions
Body: HashMap<String, f64>  (chiave → nuovo valore)
→ Assunzioni aggiornate, KPI ricalcolati per tutti i prodotti
```

### Churn

```
GET /api/churn
→ { "action": [f64; 42], "no_action": [f64; 42] }

POST /api/churn
Body: { "action": [...], "no_action": [...] }
→ Curve aggiornate, NPV incrementale ricalcolato
```

### Export

```
POST /api/export/economics
→ File xlsx (stream) — economics output

POST /api/export/fasce
→ File xlsx — fasce output

POST /api/export/fasce_request
→ File xlsx — fasce request

POST /api/export/fasce_config
→ File xlsx — fasce config

POST /api/export/full-package
→ File ZIP — tutti gli export bundled

POST /api/export/compare
Body: { "file_a": "path_or_name", "file_b": "path_or_name" }
→ { "diffs": [{ "sheet", "cell", "value_a", "value_b" }] }
```

### Upload

```
POST /api/upload
Content-Type: multipart/form-data
Field "file": file Excel (.xlsx, .xlsb)
→ { "filename": "..._UPLOADED.xlsx", "path": "data/inputs/..." }
Dopo l'upload, il PricingEngine viene ricaricato automaticamente.
```

### Compliance

```
POST /api/compliance
Body: { "tolerance": 0.01 }
→ ComplianceResult {
    score_globale: f64,
    checked: usize,
    passed: usize,
    per_mode: { "VAR_CC": { score, checked, passed }, ... },
    per_kpi:  { "npv": ..., "pb_pl": ..., "pb_cash": ... },
    mismatches: [{ product_id, mode, kpi, expected, actual, delta }]
  }
```

---

## 8. Build e rilascio

### Build locale (macOS, sviluppo)

```bash
cargo build           # debug — binario in target/debug/tid
cargo build --release # release — binario in target/release/tid (~4MB)
cargo run -- serve    # avvia server su :5002
```

Il profilo release usa: `opt-level=3`, `lto=true`, `codegen-units=1`, `strip=true`, `panic=abort`.

### Build Windows (CI)

La build Windows avviene **solo via GitHub Actions** su runner `windows-latest`. La cross-compilazione da macOS non funziona (mancano gli MSVC sysroot headers).

**Trigger**: push di un tag `v*` → `build-release.yml` si attiva automaticamente.

**Flag critici**:
```bash
RUSTFLAGS="-C target-feature=+crt-static"
```
Questo produce un exe completamente statico — nessuna dipendenza dalla MSVC runtime (ucrtbase.dll, vcruntime.dll). L'utente non deve installare nulla.

### Struttura del ZIP di release

```
tiD-v1.0.0-windows-x64.zip
└── tiD/
    ├── tiD.exe
    ├── START_tiD.bat
    ├── README.md
    ├── frontend/dist/
    │   ├── index.html
    │   └── assets/
    └── data/
        ├── sources/   ← 4 file Excel (~35MB)
        ├── inputs/    ← vuoto
        └── exports/   ← vuoto
```

### Come pubblicare una nuova release

```bash
# 1. Assicurarsi che cargo test passi
cargo test

# 2. Aggiornare versione in Cargo.toml [package] version
# 3. Aggiornare PLAN.md progress log
# 4. Committare

git add -A
git commit -m "release: vX.Y.Z — descrizione"

# 5. Taggare
git tag vX.Y.Z -m "Release vX.Y.Z"
git push origin main
git push origin vX.Y.Z
# → CI builda e crea la release su GitHub automaticamente
```

---

## 9. Test

### Test unitari (18 test)

```bash
cargo test                        # tutti
cargo test economics              # solo economics_test.rs
```

I test sono in `tests/economics_test.rs` e coprono:

- `test_cashflow_24m/30m/36m` — struttura cashflow per durata
- `test_npv_positive_rate/zero_rate` — calcolo NPV
- `test_payback_immediate/simple/never` — payback P&L
- `test_bad_debt_var_cc/var_rid/fin_zero` — bad debt per modo
- `test_financing_cost_compass/var_zero` — financing cost
- `test_net_arpu_cluster/cluster_cb` — Net ARPU per cluster
- `test_calculate_mode_kpis_var_cc/fin_compass` — KPI end-to-end
- `test_trunc` — troncamento decimali

### Compliance check (vs tiC baseline)

```bash
cargo run -- compliance --tolerance 0.01
# o con report markdown:
cargo run -- compliance --tolerance 0.01 --report md
```

Legge i KPI dal file Excel baseline (`output_TI_CVM.xlsb`) e li confronta con quelli calcolati da tiD. Score atteso: ≥99.9%.

**Soglia di pass nel main**: il compliance check CLI esce con codice 0 se score ≥80%, 1 altrimenti. La soglia 80% è conservativa — in produzione il target è 99.9%.

### Validazione file

```bash
cargo run -- validate
```

Verifica che i 3 file Excel factory siano presenti e accessibili. Utile per debug post-deploy.

---

## 10. Configurazione e assunzioni

Non esiste un file di configurazione esterno. Tutti i parametri vengono da:

1. **File Excel** (`ASSUMPTIONS` sheet in `Listino_CVM_ECONOMICS.xlsx`) — valori modificabili a runtime via API
2. **Costanti hardcoded in Rust** — usate come fallback se l'assunzione non è presente nel file

### Porta HTTP

Definita in `src/paths.rs`:
```rust
pub const PORT: u16 = 5002;
```

Per cambiarla: modificare la costante (richiede ricompilazione) oppure passare il flag CLI:
```bash
tid.exe serve --port 8080
```

### Variabili d'ambiente utili

| Variabile | Effetto |
|---|---|
| `RUST_LOG=debug` | Abilita log verbose (tracing) |
| `RUST_LOG=tid=trace` | Log solo del crate tid |

### Assunzioni critiche (default hardcoded)

Se il file Excel non contiene un'assunzione, tiD usa questi fallback:

| Chiave | Default | Significato |
|---|---|---|
| `BAD_DEBT_CC` | 0.081 (8.1%) | Tasso bad debt VAR_CC |
| `BAD_DEBT_RID` | 0.0936 (9.36%) | Tasso bad debt VAR_RID |
| `RATE_COMPASS` | 0.083 (8.3%) | Tasso FIN_COMPASS |
| `RATE_FINDO` | 0.0805 (8.05%) | Tasso FIN_FINDO |
| `TARGET_PB_VAR` | 12 | Target payback VAR/RELOAD (mesi) |
| `TARGET_PB_FIN` | 16 | Target payback FIN (mesi) |
| `ACT_FEE` | 6.99 | Costo attivazione |

---

## 11. Deployment Windows

### Requisiti

- Windows 10/11 x86_64
- Nessun diritto amministratore
- Nessun runtime aggiuntivo (exe completamente statico)
- Porta 5002 non occupata da altri processi

### Struttura attesa su disco

Il file `paths.rs` risolve **tutti i path relativamente alla directory dell'exe**. Se la struttura non rispetta questo layout, l'app non trova i dati:

```
<qualsiasi cartella>/
├── tiD.exe           ← l'exe
├── frontend/dist/    ← DEVE essere qui, relativo all'exe
└── data/
    └── sources/      ← DEVE contenere i file Excel
```

### START_tiD.bat

```bat
@echo off
start "" "http://127.0.0.1:5002"
tiD.exe serve
pause
```

Apre il browser e avvia il server. Il `pause` finale mantiene la finestra CMD aperta (utile per vedere errori).

### Aggiornare i file Excel in produzione

L'utente ha due opzioni:
1. **Via UI** — caricare il file tramite il bottone "Carica file". Il file viene salvato in `data/inputs/` con suffisso `_UPLOADED` e il sistema ricarica automaticamente.
2. **Manuale** — copiare il file in `data/inputs/` e riavviare il tool.

I file in `data/inputs/` hanno priorità su quelli in `data/sources/`.

---

## 12. Troubleshooting

### L'app non si avvia / non trova i file Excel

**Sintomo**: `❌ ECONOMICS: File 'Listino_CVM_ECONOMICS.xlsx' not found in data/inputs/ or data/sources/`

**Cause possibili**:
1. I file Excel non sono in `data/sources/` accanto all'exe
2. L'exe è stato spostato senza spostare la cartella `data/`
3. (Versioni pre-v1.0) Bug paths.rs — risolto in v1.0.0

**Fix**: eseguire `tiD.exe validate` per diagnostica dettagliata. Verificare che `data/sources/` sia nella stessa cartella dell'exe.

### Il browser non carica l'interfaccia

**Sintomo**: pagina bianca o 404 su `http://127.0.0.1:5002`

**Cause possibili**:
1. `frontend/dist/` mancante accanto all'exe
2. Il server non è avviato (controllare la finestra CMD)
3. La porta 5002 è occupata → riavviare il tool o cambiare porta

**Fix**: verificare che `frontend/dist/index.html` esista. Se la porta è occupata:
```bash
tiD.exe serve --port 8080
```

### KPI errati o compliance bassa

**Sintomo**: `cargo run -- compliance --tolerance 0.01` mostra score <99.9%

**Cause possibili**:
1. Il file Excel `output_TI_CVM.xlsb` è aggiornato ma l'engine no (o viceversa)
2. Modifica accidentale a una formula in `economics.rs`
3. Nuove assunzioni nel file Excel non mappate in `ASSUMPTION_CELLS`

**Debug**:
```bash
cargo run -- compliance --tolerance 0.01 --report md | head -50
# Mostra i primi mismatch con product_id, mode, kpi, expected, actual, delta
```

### Upload file non funziona

**Sintomo**: POST `/api/upload` restituisce errore

**Cause possibili**:
1. Il file non è `.xlsx` o `.xlsb` (formato non supportato)
2. `data/inputs/` non esiste o non è scrivibile
3. Il file è troppo grande (Axum ha un limite di 100MB di default su multipart)

**Fix**: assicurarsi che `data/inputs/` esista (viene creato da `ensure_data_dirs()` all'avvio).

### Windows: il BAT si chiude subito

**Sintomo**: `START_tiD.bat` apre una finestra che si chiude immediatamente

**Causa**: l'exe ha crashato all'avvio (tipicamente file Excel non trovati).

**Fix**: aprire CMD manualmente e lanciare `tiD.exe serve` per vedere l'errore.

### Compilazione fallisce con errori MSVC (cross-compile da macOS)

La cross-compilazione da macOS → Windows non è supportata. Usare la CI GitHub Actions.

### Cargo.lock non in git

Per scelta progettuale, `Cargo.lock` è in `.gitignore`. Per build riproducibili in produzione è preferibile tracciarlo. Valutare per v2.0.

---

## 13. Roadmap futura

Da `PLAN.md` — item deferred e idee per le versioni successive:

### v0.5 — Ottimizzatore parametrico (deferred)
Algoritmo di ottimizzazione che, dato un target NPV o payback, suggerisce automaticamente i valori ottimali di fascia/anticipo/rata. Non implementato in v1.0.

### v1.1 — Embedding frontend nel binario
Usare il crate `include_dir!` per embeddare `frontend/dist/` direttamente nell'exe. Vantaggi: distribuzione con singolo file, impossibile rompere la struttura di cartelle. Svantaggio: l'exe cresce di ~500KB e aggiornare il frontend richiede ricompilazione.

### v1.2 — Porta configurabile via env
Aggiungere supporto a `TID_PORT` env var oltre al flag CLI, per deployment in ambienti containerizzati.

### v1.3 — Smoke test Windows automatizzato
Aggiungere uno step in CI che lancia l'exe Windows, chiama `/api/health` e verifica la risposta, senza bisogno di hardware fisico.

### v2.0 — Multi-user / server mode
Attualmente tiD è single-user (in-memory, una sola istanza). Per uso multi-utente servirebbe: database (SQLite via rusqlite), sessioni, autenticazione, e un deployment su server invece che su desktop.

---

## 14. Decisioni architetturali

### Perché Rust?

La scelta Rust è motivata da un requisito non negoziabile: **singolo exe statico su Windows, zero dipendenze**. Python embeddato produce exe da 80MB+ e richiede workaround per antivirus. Go era un'alternativa valida ma l'ecosystem per calcoli finanziari è meno maturo.

### Perché non database?

tiC non usa database — tutti i dati sono in-memory caricati da Excel. tiD mantiene la stessa scelta per semplicità di deploy e per la natura del tool (single-user, session-based). Il costo: riavvio = reload completo (~200ms). Accettabile.

### Perché il frontend non è embeddato?

In v1.0, i file `frontend/dist/` sono in una sottocartella accanto all'exe (serviti da Axum via `tower_http::ServeDir`). L'embedding nel binario (con `include_dir!`) è pianificato per v1.1 ma non era necessario per l'MVP.

### Perché `current_exe()` e non `current_dir()`?

Su Windows, l'utente può lanciare il tool da una shortcut, da Esplora Risorse, o da CMD aperto in una directory diversa. In tutti questi casi, `current_dir()` punta alla working directory del processo chiamante, non alla directory dell'exe. Questo causava il crash all'avvio perché `data/` e `frontend/` non venivano trovati. Fix introdotto in v1.0.0.

### Compatibilità API con tiC

La priorità della compatibilità API garantisce che il frontend Vue (scritto per tiC) funzioni senza modifiche. Ogni deviazione dal contratto JSON di tiC richiederebbe modifiche al codice Vue, che a sua volta richiederebbe ricompilare il bundle e copiarlo in `frontend/dist/`. È un vincolo costoso da rompere — evitare.

---

*Documento generato in base al codice sorgente v1.0.0 (commit `0895a55`).*
