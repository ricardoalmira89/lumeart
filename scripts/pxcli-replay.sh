#!/usr/bin/env bash
# Envía peticiones del protocolo, una por línea, al daemon REAL de pxcli y
# escribe «<petición> => <respuesta>». Sirve para generar fixtures de
# compatibilidad (ver crates/lume-domain/tests/fixtures/).
#
# Uso: scripts/pxcli-replay.sh <archivo-de-líneas> [tamaño, por defecto 32x32]
# Necesita Go (compila reference/pxcli) y python3 (cliente del socket Unix).

set -euo pipefail
cd "$(dirname "$0")/.."

LINES="$(realpath "$1")"
SIZE="${2:-32x32}"
[ -d reference/pxcli ] || bash scripts/setup-reference.sh

BIN="target/pxcli-ref/pxcli"
if [ ! -x "$BIN" ]; then
  mkdir -p target/pxcli-ref
  (cd reference/pxcli && go build -o "../../$BIN" ./cmd/pxcli)
fi

# Los sockets Unix admiten rutas cortas: carpeta temporal propia.
DIR="$(mktemp -d)"
trap 'rm -rf "$DIR"' EXIT
"$BIN" daemon --headless --socket "$DIR/px.sock" --size "$SIZE" >"$DIR/daemon.log" 2>&1 &
for _ in $(seq 1 50); do [ -S "$DIR/px.sock" ] && break; sleep 0.1; done
[ -S "$DIR/px.sock" ] || { cat "$DIR/daemon.log" >&2; exit 1; }

python3 -I - "$DIR/px.sock" "$LINES" <<'EOF'
import socket, sys

sock_path, lines_path = sys.argv[1], sys.argv[2]

def send(line):
    s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    s.connect(sock_path)
    s.sendall((line + "\n").encode())
    data = b""
    while not data.endswith(b"\n"):
        chunk = s.recv(4096)
        if not chunk:
            break
        data += chunk
    s.close()
    return data.decode().rstrip("\n")

for line in open(lines_path, encoding="utf-8").read().split("\n"):
    if line == "" or line.strip() == "stop":
        continue
    print(f"{line} => {send(line)}")
send("stop")
EOF
