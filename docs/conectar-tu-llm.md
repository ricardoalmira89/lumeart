# Conectar LumeArt con tu LLM

LumeArt se conecta con tu agente por
[MCP](https://modelcontextprotocol.io). De momento, la instalación está
preparada y probada para **Claude Code** y **Codex**. Tu agente le pide el
arte del juego y LumeArt lo dibuja, o tu agente dibuja directamente en el
documento abierto en la ventana.

## En dos minutos

1. Instala LumeArt (ver el [README](../README.md#instalación)) y comprueba
   que `lumeart --version` funciona en una terminal.
2. Ejecuta:

   ```bash
   lumeart setup
   ```

   En los que tengas instalados (Claude Code, Codex o los dos) registra el
   servidor MCP y copia la skill (las instrucciones largas) donde cada uno la
   busca. Se puede repetir sin miedo: si ya está todo, no cambia nada
   (`--dry-run` dice qué haría).
3. En Claude Code, escribe `/mcp` (o abre una sesión nueva); en Codex, abre
   una sesión nueva. Después pídele, por ejemplo: «usa LumeArt para crear el
   sprite de un cofre de 32×32».

Para usar LumeArt desde un juego, ejecuta además `lumeart init` en su
carpeta: crea `lumeart.toml` con el estilo, el tamaño, la paleta y la carpeta
donde se guarda el arte.

## Cómo funciona

- **El servidor MCP** es el propio ejecutable: `lumeart mcp`. Habla por la
  entrada y salida estándar (stdio), lo arranca tu cliente y no necesita
  puertos ni claves.
- **Encuentra la ventana solo.** Si LumeArt está abierto, trabaja con sus
  pestañas y lo que hace tu LLM se ve al momento (y se puede deshacer). Si no,
  trabaja en una sesión propia, sin ventana.
- **Las instrucciones** llegan en dos niveles:
  - Al conectarse, el servidor le da a tu LLM unas instrucciones breves: qué
    herramientas hay y en qué orden usarlas. No hay que configurar nada.
  - La **skill** (`skills/lumeart/SKILL.md`) es la guía larga: cuándo usar
    LumeArt, la API de `dibujo.js`, cómo animar con esqueleto y cómo trabajar
    con varias pestañas. `lumeart setup` la deja en `~/.claude/skills/lumeart/`
    (Claude Code) y en `~/.agents/skills/lumeart/` (Codex), y ellos la cargan
    cuando hace falta. `lumeart setup --print skill` la imprime.

## Las dos formas de usarlo

**Encargar el arte a LumeArt** (`create_art`, `wait_art`, `revise_art`).
LumeArt lo dibuja con su propio agente, lo revisa en varias vueltas y deja el
`.png` y el `.lume` en la carpeta de arte del juego (`lumeart.toml`, que crea
`lumeart init`). Tu LLM apenas gasta: el dibujo lo paga el proveedor que
elegiste en los Ajustes de LumeArt (tu suscripción de Claude Code o una clave
de Anthropic u OpenAI). Con la ventana abierta, cada encargo sale como una
pestaña y se ve cómo avanza.

**Sin preguntar cada poco cómo va.** `wait_art` (y `wait_window_agent`, para
el agente de una pestaña) esperan hasta que el dibujo termina y, mientras
tanto, mandan avisos de progreso («vuelta 2 de 4»). En Claude Code, una
llamada que dura más de 2 minutos pasa sola a segundo plano: tu LLM sigue
con otra cosa y Claude Code le avisa cuando LumeArt termina, con las rutas
de los archivos. No hace falta hacer nada para activarlo.

**Dibujar tu LLM en el documento** (`read_vector`, `edit_vector`,
`write_vector`, `view_vector`, `write_animation`, `undo`…). Tu LLM escribe el
código del dibujo (`dibujo.js` o `anim.js`) y revisa el resultado en las
imágenes que devuelven las herramientas, así que **necesita un modelo que vea
imágenes**. Todo lo que cambia se ve en la ventana y queda en el historial.

## Varias piezas a la vez

Cada pestaña de la ventana es un documento independiente, con su propio
agente y un id fijo. Todas las herramientas del documento aceptan `tab`: con
él trabajan en esa pestaña sin cambiar la que ve el usuario; sin él, en la
activa.

Para tres piezas, tu LLM puede:

1. Crear tres pestañas con `new_document` y guardar el id que devuelve cada
   una (`list_tabs` los lista todos).
2. Trabajar en cada una con su `tab` (`write_vector`, `view_vector`…), o
   pedírselas al agente de LumeArt con `ask_window_agent` en las tres
   pestañas y esperar cada una con `wait_window_agent` (te avisa al
   terminar).
3. Guardarlas con `save_document` y su `tab`.

O, más sencillo, lanzar tres `create_art` seguidos: los tres se dibujan a la
vez.

## Registrarlo a mano

Si prefieres no usar `lumeart setup`, o quieres ver qué haría:

```bash
lumeart setup --print               # Claude Code y Codex
lumeart setup --print claude-code   # solo Claude Code
lumeart setup --print codex         # solo Codex
```

Imprime, con la ruta de `lumeart` de tu equipo, la orden que lo registra y el
bloque equivalente de su archivo de configuración, sin cambiar nada. Es esto:

**Claude Code** (para todos tus proyectos):

```bash
claude mcp add --scope user lumeart -- lumeart mcp
mkdir -p ~/.claude/skills/lumeart
lumeart setup --print skill > ~/.claude/skills/lumeart/SKILL.md
```

**Codex**:

```bash
codex mcp add lumeart -- lumeart mcp
mkdir -p ~/.agents/skills/lumeart
lumeart setup --print skill > ~/.agents/skills/lumeart/SKILL.md
```

Con otro cliente de MCP debería funcionar igual (el servidor es
`lumeart mcp` por stdio y la skill es Markdown), pero aún no lo hemos probado.

## Problemas frecuentes

- **El cliente no ve las herramientas de LumeArt.** Reinícialo después de
  cambiar su configuración y mira su registro de servidores MCP. Comprueba
  que `lumeart mcp` arranca en una terminal (se queda esperando: sal con
  Ctrl+C).
- **«No se encontró `lumeart`».** El agente no ve tu `PATH`. Usa la ruta
  completa del ejecutable: `lumeart setup` y `lumeart setup --print` ya la
  ponen si `lumeart` no está en el `PATH`.
- **Trabaja sin la ventana aunque esté abierta.** Abre LumeArt antes de pedir
  algo; el servidor la busca en cada llamada. Si abriste otra sesión con
  `lumeart --session ARCHIVO`, el cliente tiene que usar la misma
  (`lumeart --session ARCHIVO mcp`).
- **Tras reiniciar LumeArt, el cliente da errores.** Algunos clientes
  guardan la conexión vieja: reconéctalo (en Claude Code, `/mcp`).
- **«No hay pestaña N».** Los ids cambian al cerrar y abrir la ventana: pide
  otra vez `list_tabs`.
- **El encargo no termina.** `wait_art` sin `timeout_seconds` espera hasta
  el final (con un tope de tres horas). Si se le dio un `timeout_seconds` y
  dice que sigue en marcha, vuelve a llamarla sin él.
- **La espera se corta.** Claude Code corta una llamada si pasa 30 minutos
  sin noticias; LumeArt manda un aviso de progreso cada 15 segundos para que
  no ocurra. Si aun así se corta, el encargo sigue: `wait_art` con su número
  lo vuelve a esperar.
