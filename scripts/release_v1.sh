#!/usr/bin/env bash
#
# release_v1.sh — Commit finale + tag v1.0.0 per tiD
#
# Esegui da: /Volumes/HD_esterno/Progetti/tiD/
# Prerequisito: cargo test deve essere verde
#
set -euo pipefail

cd "$(dirname "$0")/.."

echo "=== tiD — Release v1.0.0 ==="
echo ""

# Verifica tests locali
echo "1/4 Running cargo test..."
cargo test
echo "✅ Tests passed"
echo ""

# Stage tutto
echo "2/4 Staging changes..."
git add \
  .github/workflows/build-release.yml \
  .gitignore \
  PLAN.md \
  README.md \
  CLAUDE.md \
  HANDOFF_MVP.md \
  frontend/dist/index.html \
  scripts/build_release.sh \
  scripts/release_v1.sh \
  src/main.rs \
  src/paths.rs

# Aggiunge i file Excel (erano git-ignored, ora no)
git add data/sources/

git status --short
echo ""

# Commit
echo "3/4 Committing..."
git commit -m "release: v1.0.0 — MVP self-contained Windows deployment

- FIX: paths.rs base_dir() usa current_exe() invece di current_dir()
  (fix critico per Windows: l'app trovava data/ e frontend/ solo se
  lanciata dalla stessa cartella dell'exe)
- FIX: frontend title 'tiC' → 'tiD'
- FEAT: data/sources/ rimosso da .gitignore — file Excel ora tracciati
- FEAT: CI include data/sources/ nel ZIP di release
- FEAT: build_release.sh include data/sources/ e START_tiD.bat
- DOCS: README riscritto per utente finale (deploy Windows, istruzioni Excel)
- DOCS: PLAN.md Phase 5 completata
"
echo "✅ Committed"
echo ""

# Tag
echo "4/4 Tagging v1.0.0..."
git tag v1.0.0 -m "Release v1.0.0 — MVP self-contained Windows deployment"
echo "✅ Tagged v1.0.0"
echo ""

echo "=== Pronto per il push ==="
echo ""
echo "Esegui per pubblicare:"
echo "  git push origin main"
echo "  git push origin v1.0.0"
echo ""
echo "GitHub Actions compilerà tiD.exe e creerà la release automaticamente."
echo "Verifica su: https://github.com/dOuReallyDo/tiD/actions"
