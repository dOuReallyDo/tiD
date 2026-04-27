# tiD — CVM Pricing Cockpit (Rust)

> **tiC reimagined in Rust** — a single self-contained Windows executable with zero dependencies.

## Quick Start

```bash
# Build (macOS/Linux for development)
cargo build

# Run server
cargo run -- serve

# Run compliance check
cargo run -- compliance --tolerance 0.01

# Validate data files
cargo run -- validate

# Build release for Windows (cross-compile)
cargo build --release --target x86_64-pc-windows-msvc
```

## Windows Deployment

1. Copy `target/x86_64-pc-windows-msvc/release/tid.exe` 
2. Copy `frontend/dist/` folder alongside the exe
3. Copy Excel files to `data/sources/`
4. Run `tid.exe serve`
5. Open `http://127.0.0.1:5002` in browser

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for full design documentation.

## Development Plan

See [PLAN.md](PLAN.md) for phase-by-phase roadmap.

## API Compatibility

tiD exposes the same REST API as tiC, ensuring the Vue frontend works without modification.

## License

MIT