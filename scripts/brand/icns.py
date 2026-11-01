#!/usr/bin/env python3
"""Genera assets/brand/icon.icns (macOS) con los PNG de assets/brand/.

Cada entrada del .icns es un PNG (macOS los admite desde 10.7), así que no
hace falta `iconutil`: solo la cabecera `icns` y, por entrada, su tipo de
cuatro letras, su longitud y los bytes del PNG.

    python3 scripts/brand/icns.py
"""

import struct
import sys
from pathlib import Path

BRAND = Path(__file__).resolve().parents[2] / "assets" / "brand"

# Tipo de entrada -> tamaño en píxeles del PNG.
ENTRIES = [
    ("icp4", 16),
    ("icp5", 32),
    ("ic11", 32),  # 16 @2x
    ("icp6", 64),
    ("ic12", 64),  # 32 @2x
    ("ic07", 128),
    ("ic08", 256),
    ("ic13", 256),  # 128 @2x
    ("ic09", 512),
    ("ic14", 512),  # 256 @2x
    ("ic10", 1024),  # 512 @2x
]


def png_size(data: bytes) -> tuple[int, int]:
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise ValueError("no es un PNG")
    return struct.unpack(">II", data[16:24])


def main() -> int:
    body = b""
    for kind, size in ENTRIES:
        data = (BRAND / f"icon-{size}.png").read_bytes()
        if png_size(data) != (size, size):
            print(f"icon-{size}.png no mide {size}x{size}", file=sys.stderr)
            return 1
        body += kind.encode("ascii") + struct.pack(">I", 8 + len(data)) + data
    icns = b"icns" + struct.pack(">I", 8 + len(body)) + body
    (BRAND / "icon.icns").write_bytes(icns)
    print(f"assets/brand/icon.icns: {len(ENTRIES)} entradas, {len(icns)} bytes")
    return 0


if __name__ == "__main__":
    sys.exit(main())
