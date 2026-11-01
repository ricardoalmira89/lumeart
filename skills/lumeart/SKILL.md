---
name: lumeart
description: Crea, revisa y anima el arte del juego con LumeArt (sprites en pixel art, ilustraciones vectoriales y sus animaciones), que lo dibuja con su propio agente o te deja trabajar sobre el documento abierto en su ventana, y lo guarda en la carpeta de arte del proyecto. Úsala siempre que el juego necesite un gráfico nuevo (personaje, objeto, icono, baldosa, fondo…), cambiar o animar uno que ya existe, o cuando el usuario hable de lo que tiene abierto en LumeArt.
---

# LumeArt

LumeArt dibuja el arte del juego con su propio agente y su bucle de
refinado. Tú **encargas** cada recurso y recibes los archivos; no dibujes
píxel a píxel salvo que el usuario lo pida.

Las herramientas son del servidor MCP `lumeart` (`lumeart setup` lo registra
en Claude Code y en Codex). Si el usuario tiene LumeArt abierto, los encargos
aparecen en su ventana; si no, se hacen sin ventana. Paga el proveedor de IA
configurado en LumeArt, no el tuyo.

## Flujo

1. **Guía de arte.** Llama a `art_guide`. Dice el estilo, el tamaño por
   defecto, la paleta, la carpeta de los archivos y la convención de nombres
   del proyecto (`lumeart.toml`).
   - Si no hay `lumeart.toml`, pregunta al usuario por el estilo y la paleta
     y créalo en la carpeta del juego, por ejemplo:
     `lumeart init --style "pixel art de 16 bits, contorno oscuro" --size 32x32 --palette pico-8`
     (paletas: `libre`, `pico-8`, `game-boy` o colores separados por comas;
     también `--output` y `--naming`).
   - Si `art_guide` da un error con números de línea, corrige ese
     `lumeart.toml`.
2. **Un encargo por recurso.** Llama a `create_art` con:
   - `description`: qué es y para qué sirve en el juego («cofre de madera que
     el jugador abre al final del nivel»);
   - `name`: nombre del archivo, sin extensión («cofre_madera»; se adapta a la
     convención de la guía);
   - opcionales: `size` («16x16», por defecto el de la guía), `view`
     (frontal, lateral, cenital, isométrica) y `style` (se suma al de la
     guía).
   - Los encargos son de `kind: sprite`. Para vector o animación, trabaja
     sobre el documento abierto (abajo).
   - Devuelve un número de encargo al momento.
3. **Espera** con `wait_art` y ese número, justo después, **sin
   `timeout_seconds`**: espera hasta que termine (puede tardar varios
   minutos) mandando avisos de progreso. En Claude Code la llamada pasa sola
   a segundo plano a los 2 minutos y te avisa cuando termina: sigue con otra
   cosa mientras tanto y **no preguntes una y otra vez**. Al terminar
   (`done`) trae las rutas del `.png` y del `.lume` y una vista previa:
   **mírala**. Con varios encargos a la vez, una `wait_art` por encargo.
4. **Revisa.** Si no encaja, llama a `revise_art` con el número o el nombre
   y `changes` en concreto («el contorno es demasiado grueso», «la tapa debe
   ser más clara»). Sigue el mismo dibujo, guarda una versión nueva y
   devuelve otro número para `wait_art`.
5. **Úsalo.** Emplea las rutas devueltas en el código del juego. No edites los
   PNG a mano: si hay que cambiarlos, `revise_art`.

`list_art` enseña los recursos que ya hay en la carpeta de la guía (con sus
versiones) y los encargos de esta sesión.

## Errores

- «ya existe»: ese nombre ya tiene archivo; usa `revise_art` o elige otro.
- `failed`: lee el mensaje (proveedor sin clave, lienzo vacío…) y díselo al
  usuario; las claves se ponen en LumeArt › Ajustes › Proveedores de IA.

## Bajo nivel

`draw` (comandos del protocolo, uno por línea: `set_pixel 1 1 red`,
`fill_rect 0 0 4 4 #8b3fe0`, `line 0 0 7 7 black`, `clear`, `new 16 16`,
`undo`…), `view_canvas` (el lienzo como imagen) y `export` (`path` y `scale`)
trabajan sobre el lienzo principal, como la CLI. Úsalos solo si el usuario
quiere dibujar a mano.

## El documento abierto y la ventana

Además de encargar, puedes hacer todo lo que se hace a mano en LumeArt sobre
el documento abierto (la pestaña activa de la ventana, o una sesión propia si
no hay ventana). Lo que cambias se ve al momento y se puede deshacer.

**Varias piezas a la vez: pestañas.** Cada pestaña es un documento
independiente con su propio agente y un id fijo (`list_tabs` los da;
`new_document` y `open_document` devuelven el de la pestaña nueva). Todas las
herramientas de abajo aceptan `tab`: con él trabajas en esa pestaña sin
cambiar la que ve el usuario; sin él, en la activa. Para tres piezas: crea
tres pestañas, guarda sus ids y trabaja en cada una con su `tab` (o lanza
`ask_window_agent` en las tres y espera cada una con `wait_window_agent`).
Los encargos de `create_art` también salen como pestañas (`list_tabs` dice su
número de encargo); léelos con `tab`, pero cámbialos con `revise_art`.

- `document`: qué hay (tamaño, paleta, historial, versiones, dibujo
  vectorial con sus capas, problemas y esqueleto, animación, pestañas).
  Llámala primero.
- **Dibujo vectorial** (`dibujo.js`): `read_vector` (código con líneas),
  `edit_vector` (cambia un trozo: `old_text` → `new_text`), `write_vector`
  (entero), `view_vector` (PNG; `crop` amplía una zona, `grid` pone
  coordenadas) y `export_vector` (.png, .svg, .pdf, .ai). Cada cambio
  devuelve los errores con su línea y una vista previa: corrígelos.
- **Animar un dibujo vectorial**: con su esqueleto. Pon cada parte que se
  mueve en `part(hueso, () => { ... })`, declara antes
  `bone(nombre, { parent, at: [x, y] })` con `at` en la articulación (la
  bisagra de una tapa, un hombro) y las acciones con
  `action(nombre, { duration, loop }, [[0, { hueso: grados }], [0.5, { hueso: { rot, x, y, scale, alpha } }], ...])`;
  `scale` (un número o `[sx, sy]`) estira o aplasta el hueso alrededor de su
  articulación (un ojo que pestañea, una onda que crece) y `alpha` (0 a 1) lo
  desvanece; los dos pasan a sus hijos.
  No cambies cómo se ve el reposo y que nada se salga del lienzo al moverse.
  Revisa con `view_vector_animation` y exporta con `export_vector_animation`
  (.png en fila o .gif).
- **Animación de pixel art** (`anim.js`): `read_animation`,
  `edit_animation`, `write_animation`, `view_animation`.
- **Historial**: `undo` (o `redo: true`), `save_version`, `restore_version`.
- **Ventana**: `list_tabs`, `new_document` (`pixel`, `vector` o
  `animation`), `open_document`, `save_document`, `select_tab` (solo cambia lo
  que ve el usuario), `close_tab`, `rename_document`, `set_mode`.
- **Su agente**: `ask_window_agent` le pide algo a la pestaña `tab` (o a la
  activa) como si lo escribiera el usuario. Justo después, `wait_window_agent`
  (mismo `tab`, sin `timeout_seconds`) espera a que termine y te avisa, como
  `wait_art`: no hace falta mirar `window_agent_status` una y otra vez. Para
  pararlo, `stop_window_agent`. Cada pestaña tiene el suyo: pueden trabajar
  varios a la vez. Úsalo si el usuario quiere que lo haga LumeArt; si no,
  hazlo tú con las herramientas de arriba.

## Sin MCP

Si las herramientas MCP no están disponibles, usa la CLI (`lumeart --help`):
`lumeart start --headless`, comandos `set_pixel`, `fill_rect`, `line`,
`batch`, `export` y `lumeart stop`.

## Buenas prácticas

- Mantén la paleta y el tamaño de la guía para que todo encaje.
- Pide primero el recurso principal (el personaje) y luego el resto a juego,
  mencionándolo en la descripción («a juego con el caballero»).
- Los `.lume` junto a cada PNG guardan su historia y sus versiones: no los
  borres.
