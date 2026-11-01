# 05 · Marca

- **Nombre**: LumeArt (familia Lume: LumeDB usa la misma forma en naranja).
- **Icono**: cuadrado redondeado (radio 22.5 %) con degradado verde a 150°
  (`#93E8A4` → `#3DBE6B` al 55 % → `#1C8F51`); bombilla color crema `#FFF5E6`
  con halo blanco al 18 %; dentro, una **paleta de pintor** con trazo `#27A35B` y
  cuatro manchas (dos verdes `#27A35B`, dos azul oscuro `#1F2A44`); casquillo de
  tres franjas `#1F2A44`.
- **Wordmark**: «LumeArt» en Geist 700, blanco sobre oscuro.
- **Acento de la interfaz**: `#3DBE6B`, texto sobre acento `#05210F`.

## Archivos (`assets/brand/`)

| Archivo | Uso |
| --- | --- |
| `icon.svg` | Fuente del icono de la app (1024×1024) |
| `icon-symbol.svg` | Solo la bombilla, sin fondo (para la cabecera o fondos claros) |
| `icon-{16,24,32,48,64,128,256,512,1024}.png` | Icono rasterizado |
| `icon.ico` | Windows (16–256): lo lleva el `.exe` (`crates/lumeart/build.rs`) |
| `icon.icns` | macOS (16–1024, entradas PNG); se regenera con `python3 scripts/brand/icns.py` |
| `../linux/lumeart.desktop` | Lanzador de Linux (`Icon=lumeart`, `StartupWMClass=lumeart`) |

La ventana lleva su icono embebido (`ViewportBuilder::with_icon`): `icon-128.png`
en Linux (en X11, el de 256 no cabe en una sola petición y se perdía) e
`icon-256.png` en Windows y macOS.
