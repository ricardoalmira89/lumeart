# Generador de referencias de rasterizado

Pinta cada caso de `crates/lume-raster/tests/fixtures/cases.json` con
`vec.Rasterize` de pxcli (Chrome sin ventana) y escribe `<caso>.png`, y en
`pxcli.json` el texto de la rejilla (`gridLines`) y el formato de los números
(`num`) para las cajas y valores de `cases.json`. Necesita Chrome o Chromium
(o `PXCLI_CHROME`).

```bash
rm -rf /tmp/pxcli-golden && cp -r reference/pxcli /tmp/pxcli-golden
cp scripts/vec-golden/export.go /tmp/pxcli-golden/internal/vec/golden_export.go
mkdir -p /tmp/pxcli-golden/cmd/vecgolden
cp scripts/vec-golden/main.go /tmp/pxcli-golden/cmd/vecgolden/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/vecgolden ./cmd/vecgolden)
/tmp/vecgolden crates/lume-raster/tests/fixtures
```

## Los `dibujo.js` (`dibujos/main.go`)

Ejecuta cada `dibujo.js` con el motor de pxcli (goja + `lib.js`) y escribe
su `pxcli.json` (tamaño, formas, avisos y hash FNV-1a de 64 bits y longitud
del SVG del studio y del limpio). Con `FULL=1`, también los SVG enteros. Sin
VectorCraft al alcance (`HOME` sin `~/.local/bin` y `PATH` sin él), como
LumeArt hasta el calco.

```bash
mkdir -p /tmp/pxcli-golden/cmd/vecdibujos
cp scripts/vec-golden/dibujos/main.go /tmp/pxcli-golden/cmd/vecdibujos/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/vecdibujos ./cmd/vecdibujos)
HOME=/nonexistent PATH=/usr/bin:/bin /tmp/vecdibujos /tmp/salida $(dirname $(ls reference/runs/*/dibujo.js))
```

Después, copia cada `/tmp/salida/<ejecución>/pxcli.json` junto a su
`dibujo.js` en `crates/lume-script/tests/fixtures/vec/`.

## `Math` de goja (`gomath/main.go`)

Escribe en bits lo que da `Math.sin`, `cos`, `tan`, `atan`, `asin`, `acos`,
`exp`, `log`, `atan2`, `pow` y `hypot` en goja para muchas entradas
(`crates/lume-script/tests/fixtures/gomath.json`). Con `CONST=1` imprime los
bits de las constantes de `crates/lume-script/src/gomath.rs`. `Exp` usa FMA
si el procesador la tiene: genéralo en uno que la tenga.

```bash
mkdir -p /tmp/pxcli-golden/cmd/gomath
cp scripts/vec-golden/gomath/main.go /tmp/pxcli-golden/cmd/gomath/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/gomath ./cmd/gomath)
/tmp/gomath > crates/lume-script/tests/fixtures/gomath.json
```

## Godot (`godot/main.go`)

Exporta cada `dibujo.js` con `vec godot` de pxcli (personaje con esqueleto
o escena por capas) dentro de un proyecto temporal y escribe su
`pxcli.json`: el tipo, el `.tscn`, el resumen de la CLI o el error, y por
cada textura su tamaño y una rejilla de 8×8 colores medios. Necesita Chrome.
Con un `HOME` sin `~/.local/bin/vectorcraft-cli` (y el `PATH` sin él), como
LumeArt hasta el calco.

```bash
mkdir -p /tmp/pxcli-golden/cmd/vecgodot
cp scripts/vec-golden/godot/main.go /tmp/pxcli-golden/cmd/vecgodot/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/vecgodot ./cmd/vecgodot)
mkdir -p /tmp/home-sin-vc
HOME=/tmp/home-sin-vc PATH=/usr/bin:/bin /tmp/vecgodot crates/lumeart/tests/fixtures/vec_godot <carpetas con dibujo.js>
```
