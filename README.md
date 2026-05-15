# tiD — CVM Pricing Cockpit

**tiD** è il tool di pricing CVM per WindTre. Singolo eseguibile Windows, zero installazione, zero dipendenze.

---

## Avvio su Windows

1. **Decomprimi** l'archivio `tiD-vX.X.X-windows-x64.zip` in una cartella a tua scelta (es. `C:\tiD\`)
2. **Doppio click** su `START_tiD.bat`
3. Si apre automaticamente il browser su `http://127.0.0.1:5002`

> Non servono diritti amministratore, Python, Node.js o nessun'altra installazione.

---

## Struttura delle cartelle

```
tiD/
├── tiD.exe               ← il tool (non spostare)
├── START_tiD.bat         ← avvio con un click
├── README.md
├── frontend/dist/        ← interfaccia web (non modificare)
└── data/
    ├── sources/          ← file Excel factory (pre-caricati)
    ├── inputs/           ← file Excel caricati dall'utente (override)
    └── exports/          ← export generati dal tool
```

---

## Caricare nuovi file Excel

Per aggiornare i dati senza toccare i file factory:

1. Apri il tool nel browser
2. Usa il pulsante **"Carica file"** nell'interfaccia
3. Seleziona il file Excel aggiornato — verrà salvato in `data/inputs/` con suffisso `_UPLOADED`
4. I dati vengono ricaricati automaticamente

In alternativa, copia manualmente i file `.xlsx` / `.xlsb` in `data/inputs/` e riavvia il tool.

---

## Export e versioning

- **Export** → genera un file Excel con formule live in `data/exports/`
- **Compare** → confronta due versioni di output cella per cella
- **Versioni** → salva e approva snapshot con label (es. `v2024-Q1`)

---

## Porta e rete

tiD ascolta solo su `127.0.0.1:5002` (loopback) — non è raggiungibile da altre macchine sulla rete per default.

---

## Per i developer

```bash
# Build locale (macOS/Linux)
cargo build --release

# Test
cargo test                              # 18 test unitari
cargo run -- compliance --tolerance 0.01   # KPI compliance check vs tiC baseline

# Build Windows (via CI)
git tag v1.x.x
git push origin v1.x.x
# → GitHub Actions compila tiD.exe statico e crea il release ZIP
```

Vedi [ARCHITECTURE.md](ARCHITECTURE.md) per il design interno e [PLAN.md](PLAN.md) per la roadmap.

**Frontend**: il bundle Vue è pre-compilato dal repo [tiC](https://github.com/dOuReallyDo/tiC). Per modifiche UI, ricompilare da tiC e copiare `dist/` in `frontend/dist/`.

---

## License

MIT
