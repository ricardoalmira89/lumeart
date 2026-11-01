#!/usr/bin/env bash
# Ejecuta `pxcli batch` (el de la CLI REAL de pxcli) con un guion contra un
# daemon nuevo sin ventana, en una carpeta temporal que se usa como directorio
# de trabajo (las rutas relativas de `export` se resuelven ahí y en la salida se
# sustituye por /BATCHDIR). Escribe la salida estándar, después
# «--- stderr ---», la salida de error y «--- exit N ---».
#
# Uso: scripts/pxcli-batch.sh <guion> [tamaño, por defecto 4x4]

set -euo pipefail
cd "$(dirname "$0")/.."

SCRIPT="$(realpath "$1")"
SIZE="${2:-4x4}"
[ -d reference/pxcli ] || bash scripts/setup-reference.sh

BIN="$(pwd)/target/pxcli-ref/pxcli"
if [ ! -x "$BIN" ]; then
  mkdir -p target/pxcli-ref
  (cd reference/pxcli && go build -o "$BIN" ./cmd/pxcli)
fi

DIR="$(mktemp -d)"
trap 'PXCLI_SOCKET="$DIR/px.sock" "$BIN" stop >/dev/null 2>&1 || true; rm -rf "$DIR"' EXIT
"$BIN" daemon --headless --socket "$DIR/px.sock" --size "$SIZE" >"$DIR/daemon.log" 2>&1 &
for _ in $(seq 1 50); do [ -S "$DIR/px.sock" ] && break; sleep 0.1; done
[ -S "$DIR/px.sock" ] || { cat "$DIR/daemon.log" >&2; exit 1; }

set +e
(cd "$DIR" && PXCLI_SOCKET="$DIR/px.sock" "$BIN" batch <"$SCRIPT" >"$DIR/out" 2>"$DIR/err")
code=$?
set -e
sed "s#$DIR#/BATCHDIR#g" "$DIR/out"
echo "--- stderr ---"
sed "s#$DIR#/BATCHDIR#g" "$DIR/err"
echo "--- exit $code ---"
