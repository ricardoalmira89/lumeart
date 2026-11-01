//! CLI de LumeArt.
//!
//! Comandos compatibles con pxcli (`set_pixel`, `fill_rect`, `line`, `clear`,
//! `get_pixel`, `export`, `undo`, `redo`, `new`, `batch`). Es un cliente fino:
//! valida lo mismo que la CLI de pxcli antes de enviar (con sus mensajes),
//! habla con la API local por HTTP e imprime lo mismo que pxcli: la respuesta
//! en la salida estándar si es `ok`, o `Error: err …` en la de error y código
//! de salida 1. `start`, `stop`, `init`, `mcp`, `setup`, `anim` y la ventana los
//! añade la raíz.
//!
//! Regla: solo depende de `lume-domain` (tipos del protocolo).

pub mod http;

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use clap::{Args, Parser, Subcommand};
use lume_domain::SessionInfo;
use lume_domain::batch::split_batch;
use lume_domain::{ArtGuide, GuideError, GuidePalette, Naming};

/// Línea de órdenes de `lumeart`.
#[derive(Debug, Parser)]
#[command(
    name = "lumeart",
    version,
    about = "LumeArt: studio de arte para videojuegos con agente de IA.",
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Archivo de estado de la sesión (por defecto, el de la carpeta de datos
    /// del usuario; también con LUMEART_SESSION). Va antes del comando.
    #[arg(long, global = true, value_name = "ARCHIVO", alias = "socket")]
    pub session: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Argumentos libres (se validan a mano para dar los mensajes de pxcli; se
/// admiten números negativos).
#[derive(Debug, Clone, Args)]
pub struct Free {
    #[arg(allow_hyphen_values = true, num_args = 0.., value_name = "ARG")]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct ExportArgs {
    /// Amplía cada píxel a N × N (1–64).
    #[arg(long, default_value_t = 1, allow_negative_numbers = true)]
    pub scale: i64,
    #[arg(allow_hyphen_values = true, num_args = 0.., value_name = "ARCHIVO.png")]
    pub args: Vec<String>,
}

/// Opciones de `start` y `serve`.
#[derive(Debug, Clone, Args)]
pub struct StartArgs {
    /// Tamaño del lienzo (WxH).
    #[arg(long, default_value = "32x32")]
    pub size: String,
    /// Sin ventana: solo la API local. Admite `--headless=true|false`, como
    /// pxcli.
    #[arg(
        long,
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "true",
        default_value_t = false,
        action = clap::ArgAction::Set
    )]
    pub headless: bool,
    /// Escala de la ventana (se acepta por compatibilidad con pxcli).
    #[arg(long, default_value_t = 10, hide = true)]
    pub scale: i64,
}

/// Opciones de `init`: los valores de la guía (lo que no se dé, por defecto).
#[derive(Debug, Clone, Default, Args)]
pub struct InitArgs {
    /// Sobrescribe el lumeart.toml que ya haya.
    #[arg(long)]
    pub force: bool,
    /// Estilo del arte en pocas palabras.
    #[arg(long, value_name = "TEXTO")]
    pub style: Option<String>,
    /// Tamaño por defecto de cada recurso (ANCHOxALTO).
    #[arg(long, value_name = "WxH")]
    pub size: Option<String>,
    /// Paleta: libre, pico-8, game-boy o colores separados por comas.
    #[arg(long, value_name = "PALETA")]
    pub palette: Option<String>,
    /// Carpeta de los archivos, relativa al proyecto (por defecto assets/art).
    #[arg(long, value_name = "CARPETA")]
    pub output: Option<String>,
    /// Nombres de archivo: snake_case, kebab-case, camelCase o PascalCase.
    #[arg(long, value_name = "CONVENCIÓN")]
    pub naming: Option<String>,
}

/// La guía que pide `init`: los valores por defecto con lo que se haya dado.
/// Los errores nombran la opción (`--size: …`).
pub fn guide_from_init(args: &InitArgs) -> Result<ArtGuide, CliError> {
    let flag = |e: GuideError| CliError(format!("--{}: {}", e.key, e.message));
    let mut guide = ArtGuide::default();
    if let Some(style) = &args.style {
        guide.style = style.trim().to_string();
    }
    if let Some(size) = &args.size {
        guide.size = ArtGuide::parse_size(size).map_err(flag)?;
    }
    if let Some(palette) = &args.palette {
        let palette = palette.trim();
        guide.palette = if palette.contains(',') || palette.starts_with('#') {
            let colors: Vec<&str> = palette
                .split(',')
                .map(str::trim)
                .filter(|c| !c.is_empty())
                .collect();
            GuidePalette::parse_colors(&colors)
        } else {
            GuidePalette::parse_name(palette)
        }
        .map_err(flag)?;
    }
    if let Some(output) = &args.output {
        guide.output = ArtGuide::parse_output(output).map_err(flag)?;
    }
    if let Some(naming) = &args.naming {
        guide.naming = Naming::parse(naming).map_err(flag)?;
    }
    Ok(guide)
}

/// `lumeart anim`: animar un dibujo (como `pxcli anim`). Trabaja con
/// archivos y no necesita una sesión abierta.
#[derive(Debug, Clone, Args)]
pub struct AnimArgs {
    #[command(subcommand)]
    pub command: AnimCommand,
}

/// Subcomandos de `anim`.
#[derive(Debug, Clone, Subcommand)]
pub enum AnimCommand {
    /// Crea anim.js a partir de un dibujo y su receta
    Init(AnimInitArgs),
    /// Dibuja todos los fotogramas y dice qué problemas hay
    Check(AnimFileArgs),
    /// Guarda una hoja para revisar: el reposo primero y una fila por acción, sobre cuadros
    Sheet(AnimSheetArgs),
    /// Guarda un fotograma
    Frame(AnimFrameArgs),
    /// Guarda la hoja de sprites (una fila por acción) y, si se pide, las tiras, el .json y el SpriteFrames de Godot
    Export(AnimExportArgs),
}

/// `anim init`.
#[derive(Debug, Clone, Args)]
pub struct AnimInitArgs {
    /// Dibujo base (PNG)
    #[arg(long, default_value = "")]
    pub from: String,
    /// Comandos de pxcli que dibujaron la base (uno por línea)
    #[arg(long, default_value = "")]
    pub recipe: String,
    /// Tamaño de la celda WxH (por defecto, el de la base)
    #[arg(long, default_value = "")]
    pub cell: String,
    /// Celda N píxeles más grande que la base
    #[arg(long, default_value_t = 0)]
    pub grow: i64,
    /// Dónde va la base en una celda más grande: bottom o center
    #[arg(long, default_value = "bottom")]
    pub anchor: String,
    /// Nombre de la animación
    #[arg(long, default_value = "")]
    pub name: String,
    /// Acción como nombre:frames:fps[:once] (se puede repetir)
    #[arg(long = "action", value_name = "ACCIÓN")]
    pub actions: Vec<String>,
    /// Archivo que se crea
    #[arg(short, long, default_value = "anim.js")]
    pub out: String,
    /// Sobrescribe un anim.js que ya exista
    #[arg(long)]
    pub force: bool,
}

/// `anim check [anim.js]`.
#[derive(Debug, Clone, Args)]
pub struct AnimFileArgs {
    #[arg(default_value = "anim.js", value_name = "anim.js")]
    pub file: String,
}

/// `anim sheet [anim.js] -o revision.png`.
#[derive(Debug, Clone, Args)]
pub struct AnimSheetArgs {
    #[arg(default_value = "anim.js", value_name = "anim.js")]
    pub file: String,
    /// PNG de salida
    #[arg(short, long, default_value = "revision.png")]
    pub out: String,
    /// Amplía cada píxel N veces
    #[arg(long, default_value_t = 8, allow_negative_numbers = true)]
    pub scale: i64,
}

/// `anim frame [anim.js] --action NOMBRE --frame N -o out.png`.
#[derive(Debug, Clone, Args)]
pub struct AnimFrameArgs {
    #[arg(default_value = "anim.js", value_name = "anim.js")]
    pub file: String,
    /// PNG de salida
    #[arg(short, long, default_value = "frame.png")]
    pub out: String,
    /// Acción (por defecto, la primera)
    #[arg(long, default_value = "")]
    pub action: String,
    /// Fotograma, desde 0
    #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
    pub frame: i64,
    /// Amplía cada píxel N veces
    #[arg(long, default_value_t = 1, allow_negative_numbers = true)]
    pub scale: i64,
}

/// `anim export [anim.js]`.
#[derive(Debug, Clone, Args)]
pub struct AnimExportArgs {
    #[arg(default_value = "anim.js", value_name = "anim.js")]
    pub file: String,
    /// Hoja de salida (por defecto, <nombre>_<W>x<H>.png junto a anim.js)
    #[arg(short, long, default_value = "")]
    pub out: String,
    /// Amplía cada píxel N veces
    #[arg(long, default_value_t = 1, allow_negative_numbers = true)]
    pub scale: i64,
    /// Guarda también una tira PNG por acción
    #[arg(long)]
    pub strips: bool,
    /// Guarda también los datos de la hoja (.json)
    #[arg(long)]
    pub json: bool,
    /// Guarda también un SpriteFrames de Godot 4 (.tres)
    #[arg(long)]
    pub godot: bool,
    /// Exporta aunque anim.js tenga errores
    #[arg(long)]
    pub force: bool,
}

/// `vec`: ilustraciones vectoriales (dibujo.js -> SVG y PNG).
#[derive(Debug, Clone, Args)]
pub struct VecArgs {
    #[command(subcommand)]
    pub command: VecCommand,
}

/// Subcomandos de `vec`.
#[derive(Debug, Clone, Subcommand)]
pub enum VecCommand {
    /// Crea un dibujo.js vacío
    Init(VecInitArgs),
    /// Ejecuta dibujo.js y dice qué problemas hay
    Check(VecFileArgs),
    /// Convierte dibujo.js en PNG (o solo una zona ampliada, o con una rejilla de coordenadas)
    Render(VecRenderArgs),
    /// Guarda el dibujo como un archivo SVG
    Svg(VecSvgArgs),
    /// Muestra la imagen de referencia sobre el lienzo del dibujo, con su rejilla
    Ref(VecRefArgs),
    /// Pone el dibujo junto a la imagen de referencia (o encima)
    Compare(VecCompareArgs),
    /// Exporta a Godot: un personaje con esqueleto (Skeleton2D, Bone2D, Sprite2D, AnimationPlayer) o una escena (un Parallax2D por capa, un Sprite2D por elemento)
    Godot(VecGodotArgs),
    /// Dibuja cada acción de un personaje con esqueleto como una fila de fotogramas (como la reproducirá Godot)
    Poses(VecPosesArgs),
    /// Vista previa del calco de la referencia (trace()) y la lista de sus colores
    Calco(VecCalcoArgs),
    /// Guarda el dibujo para editarlo en otro programa: .pdf, .ai, .svg y, con VectorCraft, .vectorcraft o .eps; con capas y objetos con nombre
    Export(VecExportArgs),
}

/// `vec calco -o calco.png`.
#[derive(Debug, Clone, Args)]
pub struct VecCalcoArgs {
    /// PNG que se escribe
    #[arg(short, long, default_value = "calco.png")]
    pub out: String,
    /// Imagen de referencia (por defecto, la referencia-* más nueva de aquí)
    #[arg(long = "ref", default_value = "")]
    pub reference: String,
    /// dibujo.js que da el tamaño del lienzo
    #[arg(long, default_value = "dibujo.js")]
    pub file: String,
    /// color o lines
    #[arg(long, default_value = "color")]
    pub mode: String,
    /// Colores del calco (2-64)
    #[arg(long, default_value_t = 16, allow_negative_numbers = true)]
    pub colors: i64,
    /// Fidelidad de los trazos (0-100)
    #[arg(long, default_value_t = 60.0, allow_negative_numbers = true)]
    pub detail: f64,
    /// Ignora manchas más pequeñas que esto (px)
    #[arg(long, default_value_t = 6.0, allow_negative_numbers = true)]
    pub noise: f64,
    /// Modo lines: umbral de oscuridad (0-255)
    #[arg(long, default_value_t = 110, allow_negative_numbers = true)]
    pub threshold: i64,
    /// Ancho de salida en píxeles
    #[arg(long, default_value_t = 1024, allow_negative_numbers = true)]
    pub width: i64,
}

/// `vec export [dibujo.js] -o dibujo.pdf`.
#[derive(Debug, Clone, Args)]
pub struct VecExportArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// Archivo que se escribe; su extensión elige el formato
    #[arg(short, long, default_value = "dibujo.vectorcraft")]
    pub out: String,
}

/// `vec godot [dibujo.js] -o carpeta/del/juego --name nombre`.
#[derive(Debug, Clone, Args)]
pub struct VecGodotArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// Carpeta dentro del proyecto de Godot
    #[arg(short, long, default_value = ".")]
    pub out: String,
    /// Nombre de la escena (por defecto, personaje o escena)
    #[arg(long, default_value = "")]
    pub name: String,
    /// Píxeles de textura por unidad del dibujo (0 = un personaje llena una pantalla de 1080p; una escena conserva su tamaño)
    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    pub scale: f64,
}

/// `vec poses [dibujo.js] -o poses.png`.
#[derive(Debug, Clone, Args)]
pub struct VecPosesArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// PNG que se escribe
    #[arg(short, long, default_value = "poses.png")]
    pub out: String,
    /// Solo esta acción
    #[arg(long, default_value = "")]
    pub action: String,
    /// Fotogramas por acción
    #[arg(long, default_value_t = 8, allow_negative_numbers = true)]
    pub frames: i64,
    /// Ancho de salida en píxeles
    #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
    pub width: i64,
}

/// `vec init`.
#[derive(Debug, Clone, Args)]
pub struct VecInitArgs {
    /// Tamaño del dibujo ANCHOxALTO en unidades
    #[arg(long, default_value = "1024x1024")]
    pub size: String,
    /// Empieza con el fondo transparente
    #[arg(long)]
    pub transparent: bool,
    /// Archivo que se crea
    #[arg(short, long, default_value = "dibujo.js")]
    pub out: String,
    /// Sobrescribe un archivo que ya exista
    #[arg(long)]
    pub force: bool,
}

/// `vec check [dibujo.js]`.
#[derive(Debug, Clone, Args)]
pub struct VecFileArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
}

/// `vec render [dibujo.js] -o revision.png`.
#[derive(Debug, Clone, Args)]
pub struct VecRenderArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// PNG que se escribe
    #[arg(short, long, default_value = "revision.png")]
    pub out: String,
    /// Ancho de salida en píxeles
    #[arg(long, default_value_t = 1024, allow_negative_numbers = true)]
    pub width: i64,
    /// Solo esta zona, como x,y,ancho,alto en unidades del dibujo (para ver detalles)
    #[arg(long, default_value = "", allow_hyphen_values = true)]
    pub crop: String,
    /// Rejilla de coordenadas con su valor cada N unidades
    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    pub grid: f64,
    /// Color detrás de un dibujo transparente, por ejemplo #ffffff
    #[arg(long, default_value = "")]
    pub bg: String,
}

/// `vec svg [dibujo.js] -o dibujo.svg`.
#[derive(Debug, Clone, Args)]
pub struct VecSvgArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// SVG que se escribe
    #[arg(short, long, default_value = "dibujo.svg")]
    pub out: String,
}

/// `vec ref [imagen] -o ref.png`.
#[derive(Debug, Clone, Args)]
pub struct VecRefArgs {
    /// Imagen de referencia (por defecto, la referencia-* más nueva de aquí)
    pub image: Option<String>,
    /// PNG que se escribe
    #[arg(short, long, default_value = "ref.png")]
    pub out: String,
    /// Imagen de referencia (por defecto, la referencia-* más nueva de aquí)
    #[arg(long = "ref", default_value = "")]
    pub reference: String,
    /// dibujo.js que da el tamaño del lienzo
    #[arg(long, default_value = "dibujo.js")]
    pub file: String,
    /// Ancho de salida en píxeles
    #[arg(long, default_value_t = 1024, allow_negative_numbers = true)]
    pub width: i64,
    /// Paso de la rejilla en unidades del dibujo (0 = sin rejilla)
    #[arg(long, default_value_t = 100.0, allow_negative_numbers = true)]
    pub grid: f64,
}

/// `vec compare [dibujo.js] -o comparacion.png`.
#[derive(Debug, Clone, Args)]
pub struct VecCompareArgs {
    #[arg(default_value = "dibujo.js", value_name = "dibujo.js")]
    pub file: String,
    /// PNG que se escribe
    #[arg(short, long, default_value = "comparacion.png")]
    pub out: String,
    /// Imagen de referencia (por defecto, la referencia-* más nueva de aquí)
    #[arg(long = "ref", default_value = "")]
    pub reference: String,
    /// lado (uno junto al otro) o encima (superpuestos)
    #[arg(long, default_value = "lado")]
    pub mode: String,
    /// Ancho de salida en píxeles
    #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
    pub width: i64,
    /// Paso de la rejilla en unidades del dibujo (0 = sin rejilla)
    #[arg(long, default_value_t = 0.0, allow_negative_numbers = true)]
    pub grid: f64,
}

/// `eval`: compara el bucle de LumeArt con el plugin oficial.
#[derive(Debug, Clone, Args)]
pub struct EvalArgs {
    #[command(subcommand)]
    pub command: EvalCommand,
}

/// Subcomandos de `eval`.
#[derive(Debug, Clone, Subcommand)]
pub enum EvalCommand {
    /// Hace los encargos con cada variante (A: plugin oficial, B: contexto vacío, C: misma conversación) y los deja listos para compararlos a ciegas
    Run(EvalRunArgs),
    /// Comparación ciega por parejas: eliges la mejor imagen de cada par y al final se revela qué variante era cada una
    Compare(EvalDirArgs),
    /// Escribe el informe: nota del crítico, % que pasa las comprobaciones, vueltas, tokens y tiempo (y las preferencias de la comparación)
    Report(EvalReportArgs),
}

/// `eval run -o evaluacion`.
#[derive(Debug, Clone, Args)]
pub struct EvalRunArgs {
    /// Carpeta de la evaluación (si ya existe, sigue donde se quedó)
    #[arg(short, long, default_value = "evaluacion")]
    pub out: String,
    /// Conjunto de encargos en JSON (por defecto, los 20 de LumeArt)
    #[arg(long, default_value = "")]
    pub tasks: String,
    /// Variantes: A, B, C o una lista como A,B
    #[arg(long, default_value = "A,B,C")]
    pub variants: String,
    /// Solo los N primeros encargos (0 = todos)
    #[arg(long, default_value_t = 0)]
    pub limit: usize,
    /// Vueltas como mucho, las mismas para todas (0 = las de Ajustes)
    #[arg(long, default_value_t = 0)]
    pub turns: u32,
    /// Modelo (por defecto, el de Ajustes)
    #[arg(long, default_value = "")]
    pub model: String,
    /// Carpeta del plugin ralph-loop (por defecto, el instalado en Claude Code)
    #[arg(long, default_value = "")]
    pub plugin: String,
    /// Sin pasar el crítico por cada resultado
    #[arg(long)]
    pub no_critic: bool,
    /// Semilla de las claves y del orden de la comparación (0 = una al azar)
    #[arg(long, default_value_t = 0)]
    pub seed: u64,
}

/// Una carpeta de evaluación.
#[derive(Debug, Clone, Args)]
pub struct EvalDirArgs {
    /// Carpeta de la evaluación
    #[arg(default_value = "evaluacion")]
    pub dir: String,
}

/// `eval report`.
#[derive(Debug, Clone, Args)]
pub struct EvalReportArgs {
    /// Carpeta de la evaluación
    #[arg(default_value = "evaluacion")]
    pub dir: String,
    /// Escribirlo aunque falten pares por comparar (el informe dice qué variante es cada una)
    #[arg(long)]
    pub force: bool,
}

/// Opciones de `setup`.
#[derive(Debug, Clone, Default, Args)]
pub struct SetupArgs {
    /// Solo dice qué haría, sin cambiar nada.
    #[arg(long)]
    pub dry_run: bool,
    /// No cambia nada: imprime cómo registrarlo a mano en `claude-code` o
    /// `codex` (los dos si no se dice cuál), o `skill` para imprimir las
    /// instrucciones para tu LLM.
    #[arg(long, value_name = "CLIENTE", num_args = 0..=1, default_missing_value = "todos")]
    pub print: Option<String>,
}

/// Comandos de dibujo y utilidades, compatibles con pxcli.
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Crea lumeart.toml, la guía de arte del juego, en la carpeta actual
    Init(InitArgs),
    /// Servidor MCP por la entrada y salida estándar (para Claude Code, Codex…)
    Mcp(Free),
    /// Registra el servidor MCP y copia la skill en Claude Code y Codex
    Setup(SetupArgs),
    /// Anima un dibujo: anim.js -> hoja de sprites PNG
    Anim(AnimArgs),
    /// Ilustraciones vectoriales: dibujo.js -> SVG y PNG
    Vec(VecArgs),
    /// Evalúa el bucle: los mismos encargos con el plugin oficial y con el bucle de LumeArt, y una comparación ciega
    Eval(EvalArgs),
    /// Abre una sesión en segundo plano e imprime su PID
    Start(StartArgs),
    /// Cierra la sesión abierta
    Stop(Free),
    /// (interno) Sirve una sesión en este proceso; lo lanza `start`
    #[command(hide = true)]
    Serve(StartArgs),
    /// Pinta un píxel: set_pixel <x> <y> <color>
    #[command(name = "set_pixel")]
    SetPixel(Free),
    /// Rellena un rectángulo: fill_rect <x> <y> <w> <h> <color>
    #[command(name = "fill_rect")]
    FillRect(Free),
    /// Dibuja una línea: line <x1> <y1> <x2> <y2> <color>
    Line(Free),
    /// Borra el lienzo: clear [color]
    Clear(Free),
    /// Lee un píxel: get_pixel <x> <y>
    #[command(name = "get_pixel")]
    GetPixel(Free),
    /// Exporta el lienzo a PNG: export [--scale N] <archivo.png>
    Export(ExportArgs),
    /// Deshace el último cambio
    Undo(Free),
    /// Rehace el último cambio deshecho
    Redo(Free),
    /// Lienzo nuevo en blanco: new <W>x<H> | new <W> <H>
    New(Free),
    /// Varios comandos de una vez (uno por línea o separados por `;`), como
    /// argumento o por la entrada estándar
    Batch(Free),
}

/// Entorno en el que se ejecuta un comando (inyectable en tests).
pub struct Context<'a> {
    /// Carpeta donde se ejecuta el comando (para `export` y `batch`).
    pub cwd: PathBuf,
    /// Valor de `LUMEART_SESSION`.
    pub session_env: Option<String>,
    /// Valor de `LUMEART_RUN`: los comandos los manda el agente en esa
    /// ejecución.
    pub agent_run: Option<u64>,
    /// Archivo de estado por defecto.
    pub default_session: Option<PathBuf>,
    pub stdin: &'a mut dyn Read,
    pub stdout: &'a mut dyn Write,
    pub stderr: &'a mut dyn Write,
}

/// Un error que la CLI imprime como `Error: <mensaje>` y sale con 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError(pub String);

fn invalid_args(message: impl AsRef<str>) -> CliError {
    CliError(format!("err invalid_args {}", message.as_ref()))
}

fn arg_count(expected: usize, got: usize) -> CliError {
    invalid_args(format!("expected {expected} args, got {got}"))
}

/// Entero como `strconv.Atoi` (lo que valida la CLI de pxcli).
fn int(value: &str, name: &str) -> Result<i64, CliError> {
    value
        .parse::<i64>()
        .map_err(|_| invalid_args(format!("{name} must be an integer")))
}

fn expect(args: &[String], n: usize) -> Result<(), CliError> {
    if args.len() != n {
        return Err(arg_count(n, args.len()));
    }
    Ok(())
}

/// Qué hay que enviar a la API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outgoing {
    /// Una línea del protocolo.
    Line(String),
    /// Un guion de `batch`.
    Batch(String),
}

/// Valida un comando como la CLI de pxcli y construye lo que se envía.
pub fn build_request(
    command: &Command,
    cwd: &Path,
    stdin: &mut dyn Read,
) -> Result<Outgoing, CliError> {
    let line = |s: String| Ok(Outgoing::Line(s));
    match command {
        Command::SetPixel(Free { args }) => {
            expect(args, 3)?;
            int(&args[0], "x")?;
            int(&args[1], "y")?;
            line(format!("set_pixel {} {} {}", args[0], args[1], args[2]))
        }
        Command::FillRect(Free { args }) => {
            expect(args, 5)?;
            int(&args[0], "x")?;
            int(&args[1], "y")?;
            let w = int(&args[2], "w")?;
            let h = int(&args[3], "h")?;
            if w <= 0 {
                return Err(invalid_args("w must be > 0"));
            }
            if h <= 0 {
                return Err(invalid_args("h must be > 0"));
            }
            line(format!("fill_rect {}", args.join(" ")))
        }
        Command::Line(Free { args }) => {
            expect(args, 5)?;
            for (value, name) in args.iter().zip(["x1", "y1", "x2", "y2"]) {
                int(value, name)?;
            }
            line(format!("line {}", args.join(" ")))
        }
        Command::Clear(Free { args }) => {
            if args.len() > 1 {
                return Err(arg_count(1, args.len()));
            }
            match args.first() {
                Some(color) => line(format!("clear {color}")),
                None => line("clear".into()),
            }
        }
        Command::GetPixel(Free { args }) => {
            expect(args, 2)?;
            int(&args[0], "x")?;
            int(&args[1], "y")?;
            line(format!("get_pixel {} {}", args[0], args[1]))
        }
        Command::Export(ExportArgs { scale, args }) => {
            expect(args, 1)?;
            let path = absolute(cwd, &args[0]);
            if !(1..=64).contains(scale) {
                return Err(invalid_args("scale must be between 1 and 64"));
            }
            let mut request = format!(
                "export {}",
                lume_domain::protocol::quote_field(&path.to_string_lossy())
            );
            if *scale > 1 {
                request.push_str(&format!(" {scale}"));
            }
            line(request)
        }
        Command::Undo(Free { args }) => {
            expect(args, 0)?;
            line("undo".into())
        }
        Command::Redo(Free { args }) => {
            expect(args, 0)?;
            line("redo".into())
        }
        Command::New(Free { args }) => {
            let (w, h) = match args.as_slice() {
                [size] => parse_canvas_size(size)?,
                [w, h] => (int(w, "width")?, int(h, "height")?),
                _ => return Err(invalid_args("expected WxH or W H")),
            };
            line(format!("new {w} {h}"))
        }
        Command::Start(_)
        | Command::Stop(_)
        | Command::Serve(_)
        | Command::Init(_)
        | Command::Mcp(_)
        | Command::Setup(_)
        | Command::Anim(_)
        | Command::Vec(_)
        | Command::Eval(_) => Err(CliError(
            "err invalid_command start, stop, serve, init, mcp, setup, anim, vec and eval are handled by lumeart itself"
                .into(),
        )),
        Command::Batch(Free { args }) => {
            let script = if args.is_empty() {
                let mut text = String::new();
                stdin
                    .read_to_string(&mut text)
                    .map_err(|e| CliError(format!("err io {e}")))?;
                text
            } else {
                args.join("\n")
            };
            if split_batch(&script).is_empty() {
                return Err(invalid_args("batch is empty"));
            }
            Ok(Outgoing::Batch(script))
        }
    }
}

/// `parseCanvasSize` de pxcli (sus errores no llevan `err`).
pub fn parse_canvas_size(raw: &str) -> Result<(i64, i64), CliError> {
    let bad = || CliError(format!("invalid size {raw:?}: expected WxH"));
    let lower = raw.trim().to_lowercase();
    let (w, h) = lower.split_once('x').ok_or_else(bad)?;
    if h.contains('x') || w.is_empty() || h.is_empty() {
        return Err(bad());
    }
    let w: i64 = w.parse().map_err(|_| bad())?;
    let h: i64 = h.parse().map_err(|_| bad())?;
    if w <= 0 || h <= 0 {
        return Err(CliError(format!(
            "invalid size {raw:?}: width and height must be positive"
        )));
    }
    Ok((w, h))
}

/// Ruta absoluta respecto a `cwd` (como `filepath.Abs`, sin tocar el disco).
pub fn absolute(cwd: &Path, path: &str) -> PathBuf {
    let path = Path::new(path);
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    let mut out = PathBuf::new();
    for part in joined.components() {
        match part {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Archivo de estado elegido: `--session`, `LUMEART_SESSION` o el de por
/// defecto.
pub fn session_path(flag: Option<&Path>, ctx: &Context<'_>) -> Result<PathBuf, CliError> {
    let chosen = match flag {
        Some(path) => Some(path.to_path_buf()),
        None => match ctx.session_env.as_deref().map(str::trim) {
            Some(env) if !env.is_empty() => Some(PathBuf::from(env)),
            _ => ctx.default_session.clone(),
        },
    };
    match chosen {
        Some(path) if !path.as_os_str().is_empty() => Ok(path),
        _ => Err(invalid_args("session path must not be empty")),
    }
}

fn not_running(path: &Path) -> CliError {
    CliError(format!(
        "err daemon_not_running no LumeArt session is running ({} not found); start one with \"lumeart start\"",
        path.display()
    ))
}

/// Lee el archivo de estado de la sesión.
pub fn read_session(path: &Path) -> Result<SessionInfo, CliError> {
    let bytes = std::fs::read(path).map_err(|_| not_running(path))?;
    serde_json::from_slice(&bytes).map_err(|e| {
        CliError(format!(
            "err daemon_not_running invalid session file {}: {e}",
            path.display()
        ))
    })
}

/// Petición a la API de una sesión.
pub fn call(
    info: &SessionInfo,
    path: &Path,
    route: &str,
    body: &str,
) -> Result<http::HttpResponse, CliError> {
    http::request(info.port, &info.token, "POST", route, body.as_bytes()).map_err(|e| match e {
        http::HttpError::Connect(_) => CliError(format!(
            "err daemon_not_running the session in {} does not answer on port {}; start one with \"lumeart start\"",
            path.display(),
            info.port
        )),
        http::HttpError::Io(m) => CliError(format!("err io {m}")),
    })
}

/// Ejecuta un comando de cliente. Devuelve el código de salida.
pub fn execute(command: &Command, session_flag: Option<&Path>, ctx: &mut Context<'_>) -> i32 {
    match try_execute(command, session_flag, ctx) {
        Ok(code) => code,
        Err(CliError(message)) => {
            let _ = writeln!(ctx.stderr, "Error: {message}");
            1
        }
    }
}

fn try_execute(
    command: &Command,
    session_flag: Option<&Path>,
    ctx: &mut Context<'_>,
) -> Result<i32, CliError> {
    let outgoing = build_request(command, &ctx.cwd, ctx.stdin)?;
    let path = session_path(session_flag, ctx)?;
    let info = read_session(&path)?;
    let run = ctx
        .agent_run
        .map(|run| format!("run={run}"))
        .unwrap_or_default();
    match outgoing {
        Outgoing::Line(line) => {
            let route = if run.is_empty() {
                "/v1/commands".to_string()
            } else {
                format!("/v1/commands?{run}")
            };
            let response = call(&info, &path, &route, &line)?;
            let text = response.text();
            let text = text.trim_end_matches('\n');
            if response.status != 200 || text.starts_with("err") {
                return Err(CliError(text.to_string()));
            }
            let _ = writeln!(ctx.stdout, "{text}");
            Ok(0)
        }
        Outgoing::Batch(script) => {
            let mut route = format!(
                "/v1/commands?batch=1&cwd={}",
                http::encode_query(&ctx.cwd.to_string_lossy())
            );
            if !run.is_empty() {
                route.push('&');
                route.push_str(&run);
            }
            let response = call(&info, &path, &route, &script)?;
            let text = response.text();
            if response.status != 200 {
                return Err(CliError(text.trim_end_matches('\n').to_string()));
            }
            let _ = write!(ctx.stdout, "{text}");
            let (ok, failed) = parse_summary(&text).unwrap_or((0, 0));
            if failed > 0 {
                return Err(CliError(format!(
                    "err batch {failed} of {} commands failed",
                    ok + failed
                )));
            }
            Ok(0)
        }
    }
}

/// Lee `batch: N ok, M failed` de la última línea.
fn parse_summary(output: &str) -> Option<(usize, usize)> {
    let last = output.lines().last()?;
    let rest = last.strip_prefix("batch: ")?;
    let (ok, failed) = rest.split_once(" ok, ")?;
    let failed = failed.strip_suffix(" failed")?;
    Some((ok.parse().ok()?, failed.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Command {
        let mut full = vec!["lumeart"];
        full.extend_from_slice(args);
        Cli::try_parse_from(full).unwrap().command.unwrap()
    }

    fn build(args: &[&str]) -> Result<Outgoing, CliError> {
        build_request(&parse(args), Path::new("/trabajo"), &mut std::io::empty())
    }

    fn line(s: &str) -> Result<Outgoing, CliError> {
        Ok(Outgoing::Line(s.to_string()))
    }

    // Portado de TestDrawCommands_FormatRequests y TestHistoryCommands.
    #[test]
    fn draw_commands_format_requests() {
        assert_eq!(
            build(&["set_pixel", "1", "2", "red"]),
            line("set_pixel 1 2 red")
        );
        assert_eq!(
            build(&["fill_rect", "0", "1", "2", "3", "#ff00ff"]),
            line("fill_rect 0 1 2 3 #ff00ff")
        );
        assert_eq!(
            build(&["line", "0", "0", "3", "0", "blue"]),
            line("line 0 0 3 0 blue")
        );
        assert_eq!(build(&["clear"]), line("clear"));
        assert_eq!(build(&["clear", "transparent"]), line("clear transparent"));
        assert_eq!(build(&["undo"]), line("undo"));
        assert_eq!(build(&["redo"]), line("redo"));
        assert_eq!(build(&["get_pixel", "3", "4"]), line("get_pixel 3 4"));
    }

    #[test]
    fn negative_numbers_are_allowed() {
        assert_eq!(
            build(&["set_pixel", "-1", "10", "red"]),
            line("set_pixel -1 10 red")
        );
        assert_eq!(
            build(&["line", "-3", "0", "2", "-1", "red"]),
            line("line -3 0 2 -1 red")
        );
    }

    // Portado de TestFillRectCmd_InvalidSize y TestLineCmd_InvalidInteger.
    #[test]
    fn invalid_arguments_are_rejected_before_sending() {
        let err = |args: &[&str]| build(args).unwrap_err().0;
        assert_eq!(
            err(&["fill_rect", "0", "0", "-1", "2", "red"]),
            "err invalid_args w must be > 0"
        );
        assert_eq!(
            err(&["fill_rect", "0", "0", "1", "0", "red"]),
            "err invalid_args h must be > 0"
        );
        assert_eq!(
            err(&["line", "0", "a", "1", "2", "red"]),
            "err invalid_args y1 must be an integer"
        );
        assert_eq!(
            err(&["set_pixel", "1", "2"]),
            "err invalid_args expected 3 args, got 2"
        );
        assert_eq!(
            err(&["set_pixel", "x", "2", "red"]),
            "err invalid_args x must be an integer"
        );
        assert_eq!(
            err(&["clear", "red", "blue"]),
            "err invalid_args expected 1 args, got 2"
        );
        assert_eq!(
            err(&["get_pixel", "1"]),
            "err invalid_args expected 2 args, got 1"
        );
        assert_eq!(
            err(&["undo", "ya"]),
            "err invalid_args expected 0 args, got 1"
        );
        assert_eq!(err(&["export"]), "err invalid_args expected 1 args, got 0");
        assert_eq!(
            err(&["export", "--scale", "0", "a.png"]),
            "err invalid_args scale must be between 1 and 64"
        );
    }

    // Portado de TestExportCmd_ResolvesAbsolutePath.
    #[test]
    fn export_resolves_the_path_against_the_cwd() {
        assert_eq!(
            build(&["export", "out.png"]),
            line("export /trabajo/out.png")
        );
        assert_eq!(
            build(&["export", "--scale", "10", "arte/../sprites/./a.png"]),
            line("export /trabajo/sprites/a.png 10")
        );
        assert_eq!(
            build(&["export", "--scale=1", "/abs/b.png"]),
            line("export /abs/b.png")
        );
        assert_eq!(
            build(&["export", "mi juego/c.png"]),
            line(r#"export "/trabajo/mi juego/c.png""#)
        );
    }

    #[test]
    fn new_accepts_wxh_or_w_h() {
        assert_eq!(build(&["new", "16x8"]), line("new 16 8"));
        assert_eq!(build(&["new", "16X8"]), line("new 16 8"));
        assert_eq!(build(&["new", "16", "8"]), line("new 16 8"));
        assert_eq!(
            build(&["new", "16"]).unwrap_err().0,
            "invalid size \"16\": expected WxH"
        );
        assert_eq!(
            build(&["new", "0x8"]).unwrap_err().0,
            "invalid size \"0x8\": width and height must be positive"
        );
        assert_eq!(
            build(&["new"]).unwrap_err().0,
            "err invalid_args expected WxH or W H"
        );
        assert_eq!(
            build(&["new", "a", "8"]).unwrap_err().0,
            "err invalid_args width must be an integer"
        );
    }

    #[test]
    fn batch_from_args_or_stdin() {
        assert_eq!(
            build(&["batch", "set_pixel 0 0 red", "undo"]),
            Ok(Outgoing::Batch("set_pixel 0 0 red\nundo".into()))
        );
        let mut stdin = "fill_rect 0 0 2 2 red\n".as_bytes();
        let got = build_request(&parse(&["batch"]), Path::new("/"), &mut stdin);
        assert_eq!(got, Ok(Outgoing::Batch("fill_rect 0 0 2 2 red\n".into())));
        assert_eq!(
            build(&["batch", "# nada"]).unwrap_err().0,
            "err invalid_args batch is empty"
        );
    }

    #[test]
    fn session_path_precedence() {
        let (mut i, mut o, mut e) = (std::io::empty(), Vec::new(), Vec::new());
        let mut ctx = Context {
            cwd: PathBuf::from("/"),
            session_env: Some("/env/session.json".into()),
            agent_run: None,
            default_session: Some(PathBuf::from("/default/session.json")),
            stdin: &mut i,
            stdout: &mut o,
            stderr: &mut e,
        };
        assert_eq!(
            session_path(Some(Path::new("/flag.json")), &ctx).unwrap(),
            PathBuf::from("/flag.json")
        );
        assert_eq!(
            session_path(None, &ctx).unwrap(),
            PathBuf::from("/env/session.json")
        );
        ctx.session_env = Some("  ".into());
        assert_eq!(
            session_path(None, &ctx).unwrap(),
            PathBuf::from("/default/session.json")
        );
        assert_eq!(
            session_path(Some(Path::new("")), &ctx).unwrap_err().0,
            "err invalid_args session path must not be empty"
        );
    }

    #[test]
    fn session_flag_is_global_and_socket_is_an_alias() {
        let cli = Cli::try_parse_from(["lumeart", "--session", "/s.json", "undo"]).unwrap();
        assert_eq!(cli.session, Some(PathBuf::from("/s.json")));
        let cli = Cli::try_parse_from(["lumeart", "undo", "--socket", "/t.json"]).unwrap();
        assert_eq!(cli.session, Some(PathBuf::from("/t.json")));
    }

    #[test]
    fn without_a_session_file_the_error_is_clear() {
        let dir = std::env::temp_dir().join("lumeart-cli-sin-sesion-no-existe");
        let path = dir.join("session.json");
        let (mut i, mut o, mut e) = (std::io::empty(), Vec::new(), Vec::new());
        let mut ctx = Context {
            cwd: PathBuf::from("/"),
            session_env: None,
            agent_run: None,
            default_session: Some(path.clone()),
            stdin: &mut i,
            stdout: &mut o,
            stderr: &mut e,
        };
        let code = execute(&parse(&["set_pixel", "0", "0", "red"]), None, &mut ctx);
        assert_eq!(code, 1);
        let stderr = String::from_utf8(e).unwrap();
        assert!(
            stderr.starts_with("Error: err daemon_not_running no LumeArt session is running"),
            "{stderr}"
        );
        assert!(stderr.contains("lumeart start"));
        assert!(o.is_empty());
    }

    #[test]
    fn summary_parsing() {
        assert_eq!(
            parse_summary("line 1: x -> err y\nbatch: 3 ok, 1 failed\n"),
            Some((3, 1))
        );
        assert_eq!(parse_summary("nada"), None);
    }

    #[test]
    fn start_accepts_pxcli_flag_styles() {
        let start = |args: &[&str]| {
            let mut full = vec!["lumeart", "start"];
            full.extend_from_slice(args);
            match Cli::try_parse_from(full).unwrap().command.unwrap() {
                Command::Start(a) => (a.size, a.headless, a.scale),
                other => panic!("{other:?}"),
            }
        };
        assert_eq!(start(&[]), ("32x32".into(), false, 10));
        assert_eq!(start(&["--headless"]), ("32x32".into(), true, 10));
        assert_eq!(
            start(&["--headless=true", "--size=16x8"]),
            ("16x8".into(), true, 10)
        );
        assert_eq!(
            start(&["--size=32x32", "--headless=false", "--scale", "10"]),
            ("32x32".into(), false, 10)
        );
    }
}
