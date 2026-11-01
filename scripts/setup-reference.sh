#!/usr/bin/env bash
# Copia el prototipo pxcli a reference/ (ignorada por git) para que cada vuelta
# del ralph loop pueda leer su código, sus tests y ejecuciones reales.
#
# Busca el prototipo en $PXCLI_ROOT o, por defecto, en ../draw-pixelart.
# Uso: scripts/setup-reference.sh [--force]

set -euo pipefail
cd "$(dirname "$0")/.."

ROOT="${PXCLI_ROOT:-$(cd .. && pwd)/draw-pixelart}"
SRC="$ROOT/pixel-art-cli"
RUNS="$ROOT/pxcli-runs"
DEST="reference"

if [ ! -d "$SRC" ]; then
  echo "No encuentro el prototipo en $SRC." >&2
  echo "Indica su carpeta con PXCLI_ROOT=/ruta/a/draw-pixelart $0" >&2
  exit 1
fi

if [ -d "$DEST" ] && [ "${1:-}" != "--force" ]; then
  echo "reference/ ya existe (usa --force para rehacerla)."
  exit 0
fi

rm -rf "$DEST"
mkdir -p "$DEST/pxcli" "$DEST/runs"

# Código, tests, prompts, lib.js, guías e index.html del prototipo (sin el
# binario compilado ni la carpeta .git).
if command -v rsync >/dev/null; then
  rsync -a --exclude '.git' --exclude '/pxcli' "$SRC/" "$DEST/pxcli/"
else
  cp -R "$SRC/." "$DEST/pxcli/"
  rm -rf "$DEST/pxcli/.git" "$DEST/pxcli/pxcli"
fi

# Ejecuciones reales: solo los archivos útiles para tests de compatibilidad.
copied=0
if [ -d "$RUNS" ]; then
  for run in "$RUNS"/*/; do
    name="$(basename "$run")"
    if [ -f "$run/final.png" ] || [ -f "$run/dibujo.js" ] || [ -f "$run/anim.js" ]; then
      mkdir -p "$DEST/runs/$name"
      for f in prompt.txt meta.json recipe.txt final.png dibujo.js anim.js final.svg base.png critica-1.json; do
        [ -f "$run/$f" ] && cp "$run/$f" "$DEST/runs/$name/"
      done
      # El stream del agente sirve para probar ClaudeCodeRunner; solo si es pequeño.
      if [ -f "$run/stream.jsonl" ] && [ "$(wc -c < "$run/stream.jsonl")" -lt 2000000 ]; then
        cp "$run/stream.jsonl" "$DEST/runs/$name/"
      fi
      copied=$((copied + 1))
    fi
  done
fi

# Material extra del proyecto anterior.
[ -d "$ROOT/assets" ] && cp -R "$ROOT/assets" "$DEST/assets-antiguos"

echo "reference/ lista: código de pxcli y $copied ejecuciones en reference/runs/."
