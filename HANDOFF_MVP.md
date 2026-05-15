# tiD — Handoff MVP v1.0

**Data**: 2026-05-15
**Da**: Dou (architetto strategico)
**A**: Agente esecutore (Claude Code / sviluppatore)
**Obiettivo**: Portare tiD a MVP v1.0 deployabile su Windows — self-contained, zero-install, no admin rights.

---

## Contesto

tiD è una riscrittura in Rust di tiC (Python/Flask), il tool CVM Pricing Cockpit per WindTre.
È al 90% completato. Le fasi 1-4 sono fatte, la fase 5 (Windows packaging + release) ha 2 item rimanenti.

**Repo**: https://github.com/dOuReallyDo/tiD.git
**Path locale**: `/Volumes/HD_esterno/Progetti/tiD/`
**Ultimo commit**: `f328644` — docs: update PLAN.md — Phase 4 complete (v0.4)
**Stato engine**: 18/18 test passati, KPI compliance match con tiC ≥99.9%

---

## Cosa Funziona Già

- Engine economico completo (NPV, payback, bad debt, financing cost, ARPU, churn)
- 17 API endpoint + 1 bonus (compare) — 100% compatibili con frontend tiC
- Frontend Vue 3 pre-built in `frontend/dist/`
- CI GitHub Actions per build Windows (`build-release.yml`) — produce exe statico + ZIP
- Script avvio `START_tiD.bat` e `START_tiD.command`
- `cargo test` verde (18 test)
- `cargo build --release` su macOS — binario 3.98MB

---

## Bug Critici da Fixare

### BUG 1: Path resolution usa CWD invece di exe directory (BLOCCANTE)

**File**: `src/paths.rs`
**Problema**: `base_dir()` usa `std::env::current_dir()`. Su Windows, se l'utente lancia `START_tiD.bat` da una scorciatoia o con CWD diverso, l'app non trova `data/` né `frontend/dist/`.
**Fix**: Usare `std::env::current_exe()` e risalire alla directory parent:

```rust
pub fn base_dir() -> PathBuf {
    std::env::current_exe()
        .map(|p| p.parent().unwrap_or_else(|| p.as_path()).to_path_buf())
        .unwrap_or_else(|_| PathBuf::from("."))
}
```

**Verifica**: Su Windows, aprire cmd da `C:\` e lanciare `tiD.exe serve` passando il path completo tipo `D:\tools\tiD\tiD.exe serve`. Deve caricare dati e frontend correttamente.

---

### BUG 2: Frontend title dice "tiC" invece di "tiD"

**File**: `frontend/dist/index.html`
**Problema**: `<title>` dice "tiC - CVM Pricing Cockpit".
**Fix**: Cambiare in "tiD - CVM Pricing Cockpit".

Nota: il frontend è pre-built dal repo tiC. Per fix permanenti serve rebuildare dal source tiC (o da un fork), ma per l'MVP basta modificare il dist direttamente. È un file statico — funziona.

---

## Task MVP — Ordine di Esecuzione

### Task 1: Fix path resolution (BUG 1)
- Modificare `src/paths.rs` — `base_dir()` come sopra
- Aggiungere log trace all'avvio: `tracing::info!("Base dir: {}", base_dir().display());`
- Test: `cargo test` deve passare ancora
- Test manuale: `cargo run -- serve` da directory diversa deve funzionare

### Task 2: Fix frontend title (BUG 2)
- Editare `frontend/dist/index.html` — titolo → "tiD - CVM Pricing Cockpit"
- Verificare che il JS bundle non abbia il titolo hardcoded altrove

### Task 3: Bundle data/sources/ nel repo
- I file Excel factory (35MB) sono in `data/sources/` localmente ma git-ignored
- Opzioni:
  - **A) Rimuovere dal .gitignore e committare** (semplice, 35MB nel repo — problema per clone lento)
  - **B) Aggiungere come GitHub Release asset** (più pulito, ma l'utente deve scaricare separatamente)
  - **C) Usare Git LFS** (ottimale per binary grandi, richiede setup)
- **Raccomandazione**: Opzione A per velocità MVP. Questi file cambiano raramente. Se il repo diventa troppo pesante, migrate a LFS dopo.
- Aggiornare `.gitignore`: rimuovere le righe `data/sources/*.xlsx` e `data/sources/*.xlsb`
- Committare i 4 file: `Listino_CVM_ECONOMICS.xlsx`, `Listino_CVM_FASCE.xlsx`, `output_TI_CVM.xlsb`, `output_TI_CVM.xlsx`
- Verificare che CI funzioni: la GH Actions workflow copia `data/sources/` nella distribuzione

### Task 4: Aggiornare CI per includere data/sources
- File: `.github/workflows/build-release.yml`
- Nello step "Package distribution", aggiungere:
  ```yaml
  # Copy data sources (factory Excel files)
  if [ -d "data/sources" ] && [ "$(ls -A data/sources 2>/dev/null)" ]; then
    cp -r data/sources/* dist/tiD/data/sources/
  fi
  ```
- Verificare che l'artifact ZIP contenga i file Excel

### Task 5: Aggiornare build_release.sh per macOS
- File: `scripts/build_release.sh`
- Aggiungere copia di `data/sources/` nella dist locale, come nella CI
- Aggiungere copia di `START_tiD.bat` per consistenza

### Task 6: Smoke test su Windows
- Opzione A: usare una VM Windows (VirtualBox/UTM)
- Opzione B: usare GitHub Actions con artifact
- Procedura:
  1. Push con tag `v0.4.1` per triggerare CI
  2. Scaricare artifact `tiD-windows-x64`
  3. Decomprimere su Windows (o VM)
  4. Copiare i file Excel in `data/sources/` (se non inclusi)
  5. Doppio click su `START_tiD.bat`
  6. Verificare che il browser apra `http://127.0.0.1:5002`
  7. Verificare caricamento prodotti, navigazione, export
- **Se non si ha Windows**: cretare una GitHub Issue per smoke test e procedere con il tag v1.0

### Task 7: Aggiornare README.md per v1.0
- Sostituire il contenuto attuale (troppo tecnico/developer-oriented) con:
  - Descrizione utente: cosa fa tiD
  - Istruzioni deploy Windows: decomprimi, doppio click, browser
  - Istruzioni per caricare nuovi file Excel (data/inputs/)
  - Sezione sviluppo per contributori (rimandare ad ARCHITECTURE.md e PLAN.md)

### Task 8: Aggiornare PLAN.md
- Spuntare gli item completati nella fase 5:
  - `[x]` Smoke test on actual Windows machine (o creare issue)
  - `[x]` README with user instructions (update for v1.0)
- Aggiungere nota: BUG 1 (paths) fixato

### Task 9: Tag v1.0 release
```bash
git tag v1.0.0 -m "Release v1.0.0 — MVP self-contained Windows deployment"
git push origin v1.0.0
```
- CI builda automaticamente la release Windows
- Verificare che la release appaia su https://github.com/dOuReallyDo/tiD/releases

---

## Specifiche di Build

### Build locale (macOS, sviluppo)
```bash
cd /Volumes/HD_esterno/Progetti/tiD
cargo build --release
# Binario: target/release/tid (~4MB)
```

### Build Windows (via CI)
- Triggerata da push tag `v*`
- Runner: `windows-latest`
- Flag: `RUSTFLAGS="-C target-feature=+crt-static"` → exe completamente statico, zero runtime MSVC
- Output: `tiD-{tag}-windows-x64.zip` contenente `tiD.exe`, `frontend/dist/`, `data/sources/`, `START_tiD.bat`, `README.md`

### Cross-compilation da macOS ⚠️
**NON FUNZIONA** in mancanza di MSVC sysroot headers. Usare CI per build Windows.

---

## Struttura della Distribuzione Finale (ZIP)

```
tiD/
├── tiD.exe                    # Binario Rust statico (~4MB)
├── START_tiD.bat               # Avvio con apertura browser
├── README.md                   # Istruzioni utente
├── frontend/
│   └── dist/
│       ├── index.html
│       └── assets/
│           ├── index-*.js      # Vue SPA bundle
│           └── index-*.css
└── data/
    ├── sources/                # File Excel factory (35MB)
    │   ├── Listino_CVM_ECONOMICS.xlsx
    │   ├── Listino_CVM_FASCE.xlsx
    │   ├── output_TI_CVM.xlsb
    │   └── output_TI_CVM.xlsx
    ├── inputs/                 # (vuoto) override utente
    └── exports/                # (vuoto) output generati
```

**Dimensione totale stimata**: ~39MB (4MB exe + 35MB dati + 488KB frontend)

---

## Note Importanti

1. **Non servono diritti admin**: tiD è un singolo exe statico. Decomprimi e lancia.
2. **Non serve Python/Node**: il frontend è pre-built statico, servito dall'exe via Axum.
3. **Non serve installazione**: nessun registry, nessun PATH, nessun installer.
4. **Porta 5002**: hardcoded in `src/paths.rs` costante `PORT`. Se serve cambiarla, usare variabile d'ambiente o flag CLI.
5. **I file Excel in data/sources/ sono essenziali**: senza di essi l'app non ha dati da caricare e crasha all'avvio.
6. **Il frontend è una copia di tiC**: i sorgenti Vue sono nel repo tiC, non in tiD. Per modifiche UI, rebuildare da tiC e copiare `dist/`.

---

## Come Verificare l'MVP

Dopo aver completato tutti i task:

```bash
# 1. Build locale (macOS)
cargo build --release
./target/release/tid serve
# → Aprire http://127.0.0.1:5002 — deve caricare prodotti, consentire export

# 2. Test Rust
cargo test
# → 18/18 passati

# 3. Build CI (triggerata da tag)
git tag v1.0.0
git push origin v1.0.0
# → Verificare su GitHub Actions che la build passa
# → Scaricare l'artifact Windows
# → Decomprimere e lanciare START_tiD.bat su Windows/VM

# 4. Complience check
cargo run -- compliance --tolerance 0.01
# → Deve dare KPI score ≥99.9% rispetto a tiC
```

---

## Contatti e Riferimenti

- **Repo tiD**: https://github.com/dOuReallyDo/tiD
- **Repo tiC (reference)**: https://github.com/dOuReallyDo/tiC
- **PLAN.md**: `/Volumes/HD_esterno/Progetti/tiD/PLAN.md`
- **ARCHITECTURE.md**: `/Volumes/HD_esterno/Progetti/tiD/ARCHITECTURE.md`
- **CLAUDE.md**: non esiste (da creare per Claude Code)
- **Owner**: Dou (doureallydo@gmail.com) / Morpheus