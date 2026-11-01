# 07 · Desarrollo

Cómo se construye LumeArt y cómo se publica una versión. El README es para
quien usa LumeArt; esto, para quien lo desarrolla.

## Compilar y probar

1. **Rust** 1.95.0 (<https://rustup.rs>; `rust-toolchain.toml` lo fija y
   rustup lo instala solo al compilar aquí).
2. En Linux, para la ventana: las librerías de desarrollo de
   X11/Wayland/OpenGL de tu distribución (en Debian/Ubuntu:
   `libxkbcommon-dev libwayland-dev libxrandr-dev libxi-dev libgl1-mesa-dev`).

```bash
cargo run -p lumeart                          # el studio
cargo test --workspace                        # todos los tests
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
bash scripts/install-local.sh                 # Linux: lo instala en ~/.local
```

Las tres comprobaciones (`fmt`, `clippy` y `test`) son las de la CI y deben
pasar antes de cada commit. Los tests no abren ventanas ni usan la red; los
que necesitan una clave de API o `claude` instalado van con `#[ignore]`.

Las reglas del código (arquitectura hexagonal, un único dueño del estado,
todo cambio es un comando, secretos solo en el llavero) están en `CLAUDE.md`
(o `AGENTS.md`, el mismo texto para Codex) y en `docs/03-arquitectura.md`;
el porqué de cada decisión, en `docs/02-decisiones.md`.

## Cómo se construyó

LumeArt se construyó con un **ralph loop**: cada vuelta, una sesión nueva de
Claude Code con el contexto vacío hacía una tarea de una lista, la probaba,
hacía un commit y dejaba notas para la siguiente. Los archivos de ese bucle
(la lista de tareas, las notas, el encargo de cada vuelta, el script y los
documentos con que se diseñó, incluida una copia del prototipo pxcli en
`reference/`) son locales y no están en el repositorio; `CLAUDE.md` y
`AGENTS.md` los citan porque son las instrucciones de ese bucle.

## Qué hay aquí

| Ruta | Contenido |
| --- | --- |
| `crates/` | Workspace de Rust (11 crates) |
| `docs/` | Arquitectura, decisiones, marca, protocolo, desarrollo y cómo conectar tu LLM |
| `skills/lumeart/` | Skill para que otros agentes usen LumeArt |
| `assets/brand/` | Icono (SVG, PNG, ICO, ICNS) |
| `assets/linux/` | Lanzador de Linux (`lumeart.desktop`) |
| `scripts/` | Instalación local, generación de iconos y de los datos de prueba |
| `CLAUDE.md` / `AGENTS.md` | Reglas para los agentes que trabajan en el código |
| `.github/workflows/` | CI (`ci.yml`) y publicación (`release.yml`) |

## Publicar una versión

La publicación la hace GitHub Actions (`.github/workflows/release.yml`). Al
subir un tag con la versión, compila `lumeart` en cada plataforma y crea la
publicación con sus paquetes y un `SHA256SUMS`:

| Paquete | Qué lleva |
| --- | --- |
| `lumeart_<versión>_amd64.deb` | El ejecutable, el lanzador y los iconos (`cargo deb`, con `[package.metadata.deb]` de `crates/lumeart/Cargo.toml`) |
| `lumeart-<versión>-linux-x86_64.tar.gz` | El ejecutable, `lumeart.desktop`, `icon-256.png`, el README y las licencias |
| `lumeart-<versión>-windows-x86_64.zip` | `lumeart.exe` (con su icono), el README y las licencias |
| `lumeart-<versión>-macos-aarch64.zip` y `-x86_64.zip` | `LumeArt.app` con `icon.icns`, firmada ad hoc (no notarizada) |

Linux se compila en Ubuntu 22.04 para que funcione también con glibc más
antiguas; en macOS se comprueba que el ejecutable solo enlaza librerías del
sistema. Las versiones 0.x salen como *pre-release*.

1. Sube la versión en `[workspace.package]` de `Cargo.toml` (y
   `cargo build` para que `Cargo.lock` la recoja).
2. `git tag v0.1.0 && git push origin v0.1.0`.

Para rehacer la publicación de un tag que ya existe: **Actions → Release →
Run workflow** con ese tag.

La CI (`.github/workflows/ci.yml`) corre en cada push a `main` y en cada pull
request: formato y Clippy, y compila y prueba en Linux, Windows y macOS
(deja el ejecutable de cada uno como artefacto).

Iconos por plataforma:

| Plataforma | Cómo lleva el icono |
| --- | --- |
| Todas | La ventana lo pone al abrirse (dentro del binario: `icon-128.png` en Linux, `icon-256.png` en Windows y macOS; `app_id` `lumeart`) |
| Windows | El `.exe` lleva `icon.ico` (`crates/lumeart/build.rs`, con `winresource` al compilar en Windows) |
| Linux | `lumeart.desktop` (`Icon=lumeart`, `StartupWMClass=lumeart`) e `icon-256.png` |
| macOS | `icon.icns` (lo genera `python3 scripts/brand/icns.py` desde los PNG) |
