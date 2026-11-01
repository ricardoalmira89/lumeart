# Generador de referencias de animación

Ejecuta el motor de animación de pxcli (`internal/anim`) sobre cada
`anim.js` y escribe `pxcli.json` con los hashes de las imágenes y los avisos.
Sirve para regenerar `crates/lume-script/tests/fixtures/anim/`.

```bash
cp -r reference/pxcli /tmp/pxcli-golden
mkdir -p /tmp/pxcli-golden/cmd/golden
cp scripts/anim-golden/main.go /tmp/pxcli-golden/cmd/golden/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/golden ./cmd/golden)
/tmp/golden /tmp/salida $(dirname $(find reference/runs -name anim.js) | sort)
```

Después, copia cada `/tmp/salida/<ejecución>/pxcli.json` junto a su
`anim.js` y su `base.png` en los fixtures.

## La CLI (`cli/main.go`)

Ejecuta los pasos de cada caso de `crates/lumeart/tests/fixtures/anim_cli/`
(`steps.json`) con la CLI de animación de pxcli (`cli.NewAnimCmd()`) sobre
una copia y escribe su `pxcli_cli.json` (salida, error y archivos).

```bash
mkdir -p /tmp/pxcli-golden/cmd/animcli
cp scripts/anim-golden/cli/main.go /tmp/pxcli-golden/cmd/animcli/
(cd /tmp/pxcli-golden && GOTOOLCHAIN=go1.26.0 GOFLAGS=-mod=mod go build -o /tmp/animcli ./cmd/animcli)
/tmp/animcli crates/lumeart/tests/fixtures/anim_cli/*/
```
