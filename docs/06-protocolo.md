# 06 · Protocolo de comandos (compatible con pxcli)

LumeArt habla exactamente el protocolo de pxcli. Los agentes existentes deben
funcionar cambiando `pxcli` por `lumeart`. La fuente de verdad del
comportamiento fue el código de pxcli (`internal/protocol`, `internal/color`,
`internal/canvas`, `internal/daemon/handler.go`, `internal/cli/*`) y sus tests,
portados a los de este repositorio.

## Formato

- Texto ASCII, una petición por línea, separada por espacios en blanco.
- Respuesta de una línea: `ok`, `ok <dato>` o `err <código> <mensaje>`.

## Comandos de dibujo

| Comando | Efecto |
| --- | --- |
| `set_pixel <x> <y> <color>` | Un píxel |
| `fill_rect <x> <y> <w> <h> <color>` | Rectángulo relleno |
| `line <x1> <y1> <x2> <y2> <color>` | Línea (Bresenham, extremos incluidos) |
| `clear [color]` | Todo el lienzo (por defecto transparente) |

## Utilidades

| Comando | Efecto |
| --- | --- |
| `get_pixel <x> <y>` | `ok #rrggbbaa` |
| `export [--scale N] <archivo.png>` | Guarda PNG (ruta resuelta en el cliente) |
| `undo` / `redo` | Historial; `err no_history` si está vacío |
| `new <W>x<H>` | Lienzo nuevo, opcionalmente de otro tamaño; borra el historial |
| `batch '<comandos>'` | Varios comandos, uno por línea o separados por `;` (también por stdin). Una línea por error y al final `batch: N ok, M failed` |

Solo en LumeArt: `resize <w> <h>` cambia el tamaño conservando el dibujo
(anclado arriba a la izquierda) y se puede deshacer.

Solo en LumeArt: un argumento entre comillas dobles puede llevar espacios
(`export "/home/ana/mi juego/cofre.png" 4`); dentro, `\"` es una comilla y
`\\` una barra.

## Ciclo de vida (CLI)

| pxcli | lumeart |
| --- | --- |
| `pxcli start [--size 32x32] [--scale 10] [--headless] [--socket <ruta>]` | `lumeart start [--size 32x32] [--headless] [--session <archivo>]` (abre la ventana salvo `--headless`) |
| `pxcli stop` | `lumeart stop` |
| `PXCLI_SOCKET` | `LUMEART_SESSION` |

## Colores

- Hex: `#rgb`, `#rrggbb`, `#rrggbbaa`.
- Nombres: `black`, `white`, `red`, `green`, `blue`, `yellow`, `orange`,
  `purple`, `cyan`, `magenta`, `gray`, `grey`, `transparent` (valores exactos en
  `internal/color/color.go`).
- Formato canónico de salida: `#rrggbbaa` en minúsculas.

## Códigos de error

| Código | Cuándo |
| --- | --- |
| `invalid_command` | Comando desconocido |
| `invalid_args` | Número o tipo de argumentos incorrecto |
| `invalid_color` | Color no reconocido |
| `out_of_bounds` | Coordenada fuera del lienzo (mensaje como `pixel (-1,10) outside canvas`) |
| `no_history` | Deshacer/rehacer sin historial |
| `io` | Error al escribir el archivo |
| `renderer_unavailable` | (pxcli) ventana no disponible; en LumeArt solo si la ventana no puede abrirse |

Ejemplos: `set_pixel 10 10 #ff0000` → `ok` · `get_pixel 10 10` →
`ok #ff0000ff` · `set_pixel -1 10 #ff0000` → `err out_of_bounds pixel (-1,10) outside canvas`
(el `prd.md` de pxcli decía `x must be >= 0`, pero el código real responde así).

Los mensajes exactos se copian de pxcli; los tests de compatibilidad comparan
salidas línea a línea.
