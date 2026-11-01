# LumeArt · Instrucciones para agentes

> **Si has clonado el repositorio:** estas son las instrucciones del ralph loop
> con que se construyó LumeArt. Algunos archivos que se citan aquí son locales
> de ese bucle y no están en el repositorio: `prd.md`, `tasks.md`,
> `progress.md`, `PROMPT.md`, `docs/01-contexto-pxcli.md`,
> `docs/04-diseno-ui.md`, `docs/diseno/` y `reference/`. Las reglas del código
> sí valen para cualquiera que trabaje en él; cómo compilar y probar está en
> `docs/07-desarrollo.md`.

Estás construyendo **LumeArt**, el porte a Rust del prototipo pxcli: un studio de
arte para videojuegos con agente de IA, en un único ejecutable para Linux,
Windows y macOS. Lee `prd.md` para el producto y `docs/` para todo lo demás.

## Cómo se trabaja aquí (ralph loop)

Este repositorio se construye con un ralph loop: cada vuelta es una sesión nueva
con el contexto vacío. **Lo único que recuerdas entre vueltas es lo que está en
los archivos**: `tasks.md` (qué falta), `progress.md` (notas de las vueltas
anteriores), `git log` y el código. Por eso:

1. Lee `progress.md` y `tasks.md` antes de nada.
2. Haz **una sola tarea** por vuelta: la primera no terminada de `tasks.md`
   (salvo que `progress.md` diga que está bloqueada).
3. Antes de tocar código, lee los documentos que la tarea necesita y el código de
   referencia de pxcli que indique.
4. Escribe los tests de los criterios de aceptación, haz que pasen y verifica.
5. Marca `[x]` solo lo que esté demostrado por un test o una comprobación
   ejecutada.
6. Deja una nota breve y útil en `progress.md` para la siguiente vuelta.
7. Haz un commit. No hagas `push`.

`PROMPT.md` es el encargo exacto de cada vuelta.

## Dónde está cada cosa

| Necesito… | Mira… |
| --- | --- |
| Qué es el producto | `prd.md` |
| Por qué se decidió algo | `docs/02-decisiones.md` |
| Crates, capas, `Session`, comandos, agente, bucle, MCP | `docs/03-arquitectura.md` |
| Cómo debe verse una pantalla | `docs/04-diseno-ui.md`, `docs/diseno/capturas/*.png` (míralas), `docs/diseno/*.dc.html` (valores exactos) |
| Icono y colores de marca | `docs/05-marca.md`, `assets/brand/` |
| Protocolo, comandos y errores compatibles con pxcli | `docs/06-protocolo.md` |
| Cómo lo hacía el prototipo | `docs/01-contexto-pxcli.md` y `reference/pxcli/` |
| Ejecuciones reales para tests de compatibilidad | `reference/runs/` |
| La skill para otros agentes | `skills/lumeart/SKILL.md` |

Si `reference/` no existe, ejecuta `bash scripts/setup-reference.sh`.

## Reglas que no se rompen

- **Arquitectura hexagonal.** `lume-domain` no hace E/S y solo usa `serde` y
  `thiserror`. Los adaptadores dependen de `lume-app`/`lume-domain` y nunca entre
  sí; `lume-cli` solo de `lume-domain`. Lo comprueba
  `crates/lumeart/tests/arquitectura.rs`: **si falla, arregla la dependencia, no
  el test.**
- **Un único dueño del estado**: solo la `Session` modifica el documento. La UI
  emite `Intent`s; el agente y la CLI envían comandos.
- **Todo cambio es un `Command`** con su `Patch` (deshacer) y su `Origin`.
- **Compatibilidad con pxcli**: mismos comandos, argumentos, salidas y códigos de
  error. Si dudas, lo que hace pxcli es lo correcto.
- **Sin dependencias de sistema**: nada de Chrome, Python ni librerías que el
  usuario deba instalar. Crates en Rust puro o enlazados estáticamente.
- **El bucle de refinado empieza cada vuelta con el contexto vacío.**
- **Secretos**: las claves de API solo pasan por `SecretStore`; nunca en
  archivos, logs ni tests.
- **Los tests no abren ventanas ni usan la red.** Lo que necesita clave o
  `claude` instalado va con `#[ignore]`.
- **Nunca** borres o debilites un test para que pase, ni marques una tarea sin
  verificarla.

## Convenciones

- Rust edición 2024, toolchain estable. Dependencias externas en
  `[workspace.dependencies]` del `Cargo.toml` raíz; en cada crate,
  `nombre.workspace = true`.
- Identificadores en inglés; comentarios, documentación, textos de la UI y
  mensajes de commit en español.
- Errores con `thiserror` por crate; `anyhow` solo en `crates/lumeart`.
- Commits: `T:<n>: <título de la tarea>` (por ejemplo `T:2: Implementar Rgba`).
- Sin `unsafe` (lo prohíbe el workspace).

## Comprobaciones de cada vuelta

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Las tres deben pasar antes del commit.

## Si algo no está claro

- Busca en `docs/` y en `reference/pxcli/`.
- Si de verdad falta una decisión, toma la opción más simple coherente con los
  principios, anótala al final de `docs/02-decisiones.md` con la fecha y el
  motivo, y sigue.
- Si una tarea necesita algo que solo puede hacer el usuario (probar la ventana a
  ojo, abrir algo en Godot, una clave de API), completa todo lo demás, anota en
  `progress.md` qué debe revisar el usuario y sigue con la siguiente tarea.
