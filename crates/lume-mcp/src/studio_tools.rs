//! Herramientas del MCP sobre el documento del studio y su ventana: todo lo
//! que se hace a mano en LumeArt (el documento, el historial y las
//! versiones, el dibujo vectorial y su esqueleto, la animación de pixel art,
//! exportar, y las pestañas, los modos y el agente de la ventana).
//!
//! Trabajan con [`StudioAccess`] (el studio abierto o una sesión de este
//! proceso) y pintan aquí mismo con los motores de [`Engines`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_app::ports::{
    AnimEngine, BaseLoader, DocumentStore, PortError, RasterOptions, Rasterizer, VecEngine,
    VectorExporter,
};
use lume_app::studio::{DocKind, StudioAccess, WindowRequest, WindowState};
use lume_domain::Document;
use lume_domain::vec::VecResult;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ErrorData, schemars, tool, tool_router};
use serde::Deserialize;

use crate::{LumeMcp, fail, png_block};

/// Los motores con que el MCP pinta y exporta.
#[derive(Clone)]
pub struct Engines {
    pub vector: Arc<dyn VecEngine>,
    pub rasterizer: Arc<dyn Rasterizer>,
    pub anim: Arc<dyn AnimEngine>,
    /// PNG, GIF y `.lume`.
    pub store: Arc<dyn DocumentStore>,
    /// SVG, PDF e Illustrator.
    pub exporter: Option<Arc<dyn VectorExporter>>,
}

/// Nombres de estas herramientas, en orden alfabético.
pub const STUDIO_TOOLS: [&str; 26] = [
    "ask_window_agent",
    "close_tab",
    "document",
    "edit_animation",
    "edit_vector",
    "export_vector",
    "export_vector_animation",
    "list_tabs",
    "new_document",
    "open_document",
    "read_animation",
    "read_vector",
    "rename_document",
    "restore_version",
    "save_document",
    "save_version",
    "select_tab",
    "set_mode",
    "stop_window_agent",
    "undo",
    "view_animation",
    "view_vector",
    "view_vector_animation",
    "window_agent_status",
    "write_animation",
    "write_vector",
];

fn ok(text: impl Into<String>) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(text.into())])
}

/// El código con número de línea («  12│ …»).
fn numbered(source: &str) -> String {
    let width = source.lines().count().max(1).to_string().len();
    source
        .lines()
        .enumerate()
        .map(|(i, l)| format!("{:>width$}│ {l}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Sustituye `old` por `new` si aparece exactamente una vez.
fn replace_once(source: &str, old: &str, new: &str) -> Result<String, String> {
    match source.matches(old).count() {
        1 => Ok(source.replacen(old, new, 1)),
        0 => Err(
            "old_text no aparece en el código: cópialo tal cual (léelo antes con read_*)".into(),
        ),
        n => Err(format!(
            "old_text aparece {n} veces: añade líneas de alrededor para que sea único"
        )),
    }
}

/// Resumen de un dibujo vectorial ejecutado: tamaño, formas, capas,
/// problemas y esqueleto.
fn vector_report(res: &VecResult) -> String {
    let mut out = format!(
        "dibujo.js: {} × {} · {} formas\n",
        res.width, res.height, res.count
    );
    let layers = lume_domain::vec::studio::layers(&res.clean);
    if !layers.is_empty() {
        let list: Vec<String> = layers
            .iter()
            .map(|l| format!("{} ({})", l.name, l.shapes))
            .collect();
        out.push_str(&format!("Capas: {}\n", list.join(", ")));
    }
    let issues = lume_domain::vec::issue_lines(res);
    if issues.is_empty() {
        out.push_str("Sin problemas.\n");
    } else {
        out.push_str(&format!("Problemas:\n{}\n", issues.join("\n")));
    }
    if let Some(rig) = &res.rig {
        let bones: Vec<&str> = rig.bones.iter().map(|b| b.name.as_str()).collect();
        out.push_str(&format!(
            "Esqueleto: {} huesos ({}), {} piezas\n",
            rig.bones.len(),
            bones.join(", "),
            rig.parts.len()
        ));
        for a in &rig.actions {
            out.push_str(&format!(
                "- acción {}: {:.2} s, {} claves, {}\n",
                a.name,
                a.duration,
                a.keys.len(),
                if a.looping { "en bucle" } else { "una vez" }
            ));
        }
    }
    out
}

/// Resumen del estado de la ventana.
fn window_report(state: &WindowState) -> String {
    let mut out = String::new();
    for t in &state.tabs {
        out.push_str(&format!(
            "{} tab {} · {}{} ({}){}{}\n",
            if t.active { "▶" } else { " " },
            t.id,
            t.title,
            if t.dirty { " •sin guardar" } else { "" },
            match t.mode {
                DocKind::Pixel => "pixel art",
                DocKind::Vector => "vector",
                DocKind::Animation => "animación",
            },
            if t.agent_running {
                " · su agente está trabajando"
            } else {
                ""
            },
            match t.job {
                Some(job) => format!(" · encargo {job}"),
                None => String::new(),
            }
        ));
    }
    if let Some(tab) = state.tab {
        out.push_str(&format!("Esta respuesta es de la pestaña {tab}.\n"));
    }
    if let Some(file) = &state.file {
        out.push_str(&format!("Archivo: {file}\n"));
    }
    out.push_str(if state.agent_running {
        "Su agente está trabajando.\n"
    } else {
        "Su agente está libre.\n"
    });
    if !state.agent_messages.is_empty() {
        out.push_str("Lo último que dijo:\n");
        for m in &state.agent_messages {
            out.push_str(&format!("> {}\n", m.replace('\n', "\n> ")));
        }
    }
    if let Some(n) = &state.notice {
        out.push_str(&format!("Aviso: {n}\n"));
    }
    out
}

fn kind_of(s: &str) -> Result<DocKind, String> {
    match s.trim().to_lowercase().as_str() {
        "pixel" | "pixel_art" | "pixel art" | "sprite" => Ok(DocKind::Pixel),
        "vector" | "vectorial" => Ok(DocKind::Vector),
        "animation" | "animacion" | "animación" => Ok(DocKind::Animation),
        other => Err(format!(
            "tipo desconocido: {other} (pixel, vector o animation)"
        )),
    }
}

// --- Argumentos ------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Steps {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Cuántos pasos (1 por defecto).
    #[serde(default)]
    pub steps: Option<u32>,
    /// `true` para rehacer en vez de deshacer.
    #[serde(default)]
    pub redo: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Message {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Qué tiene esta versión («tapa más clara»).
    pub message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Number {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Número de la versión (los da `document`).
    pub number: u32,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Source {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// El código entero.
    pub source: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EditSource {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Texto que hay ahora (tal cual, una sola vez en el código).
    pub old_text: String,
    /// Texto que lo sustituye.
    pub new_text: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ViewVector {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Ancho de la imagen en píxeles (por defecto 1024).
    #[serde(default)]
    pub width: Option<u32>,
    /// Solo esta zona, [x, y, ancho, alto] en unidades del dibujo.
    #[serde(default)]
    pub crop: Option<[f64; 4]>,
    /// Rejilla con coordenadas cada N unidades (para situar cosas).
    #[serde(default)]
    pub grid: Option<f64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ViewRig {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Solo esta acción (por defecto, todas).
    #[serde(default)]
    pub action: Option<String>,
    /// Fotogramas por acción (2–24, por defecto 8).
    #[serde(default)]
    pub frames: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportVector {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Archivo de salida (relativo a tu carpeta): .png, .svg, .pdf o .ai.
    pub path: String,
    /// Ancho del PNG en píxeles (por defecto 1024).
    #[serde(default)]
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExportRig {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// La acción del esqueleto.
    pub action: String,
    /// Archivo de salida: .png (hoja de sprites, fotogramas en fila) o .gif.
    pub path: String,
    /// Fotogramas por segundo (por defecto 12).
    #[serde(default)]
    pub fps: Option<u32>,
    /// Ancho de cada fotograma en píxeles (por defecto 256).
    #[serde(default)]
    pub width: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NewDoc {
    /// `pixel`, `vector` o `animation`.
    pub kind: String,
    /// Ancho (píxeles en pixel art; unidades en vector, por defecto 1024).
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    /// Paleta fija de pixel art: `pico-8` o `game-boy` (por defecto, libre).
    #[serde(default)]
    pub palette: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocPath {
    /// Ruta del `.lume` (relativa a tu carpeta).
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MaybePath {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Dónde guardarlo (.lume, relativo a tu carpeta); si no, en su archivo.
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TabIndex {
    /// Id de la pestaña (lo da `list_tabs`).
    #[serde(default)]
    pub tab: Option<u64>,
    /// O su número de orden en la tira, desde 0.
    #[serde(default)]
    pub index: Option<usize>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct TabArg {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Name {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Nombre nuevo, sin extensión.
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ModeArg {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// `pixel`, `vector` o `animation`.
    pub mode: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Prompt {
    /// Id de la pestaña (lo dan list_tabs, new_document y open_document);
    /// sin él, la pestaña activa de la ventana.
    #[serde(default)]
    pub tab: Option<u64>,
    /// Lo que se le pide al agente de la ventana, como lo escribiría el usuario.
    pub prompt: String,
}

// --- Ayudas ----------------------------------------------------------------

impl LumeMcp {
    fn studio(&self) -> Result<&Arc<dyn StudioAccess>, CallToolResult> {
        self.studio
            .as_ref()
            .ok_or_else(|| fail("este servidor MCP no tiene acceso al studio"))
    }

    fn engines(&self) -> Result<&Engines, CallToolResult> {
        self.engines
            .as_ref()
            .ok_or_else(|| fail("este servidor MCP no puede pintar (faltan los motores)"))
    }

    fn path(&self, p: &str) -> PathBuf {
        let p = Path::new(p);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            self.cwd.join(p)
        }
    }

    async fn snapshot(&self, tab: Option<u64>) -> Result<Document, CallToolResult> {
        self.studio()?
            .snapshot(tab)
            .await
            .map_err(|e| fail(e.to_string()))
    }

    /// El `dibujo.js` del documento y su ejecución.
    async fn vector(&self, tab: Option<u64>) -> Result<(String, VecResult), CallToolResult> {
        let doc = self.snapshot(tab).await?;
        let Some(drawing) = doc.vector else {
            return Err(fail(
                "el documento no tiene dibujo vectorial: créalo con write_vector o new_document kind vector",
            ));
        };
        let res = self.engines()?.vector.render(&drawing.source);
        Ok((drawing.source, res))
    }

    async fn window(&self, tab: Option<u64>, request: WindowRequest) -> CallToolResult {
        let studio = match self.studio() {
            Ok(s) => s,
            Err(e) => return e,
        };
        match studio.window(tab, request).await {
            Ok(state) => ok(window_report(&state)),
            Err(e) => fail(e.to_string()),
        }
    }

    /// Pone el `dibujo.js` y dice cómo quedó (con una vista previa).
    async fn apply_vector(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> Result<CallToolResult, ErrorData> {
        let studio = match self.studio() {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };
        if let Err(e) = studio.set_vector(tab, source.clone()).await {
            return Ok(fail(e.to_string()));
        }
        let engines = match self.engines() {
            Ok(e) => e,
            Err(_) => return Ok(ok("dibujo.js guardado.")),
        };
        let res = engines.vector.render(&source);
        let mut blocks = vec![ContentBlock::text(format!(
            "dibujo.js guardado (es un paso que se puede deshacer).\n{}",
            vector_report(&res)
        ))];
        if let Ok(png) = render_png(engines, &res, 512, None, None) {
            blocks.push(png_block(&png));
        }
        Ok(CallToolResult::success(blocks))
    }

    /// Ejecuta `anim.js` sobre su base y dice cómo quedó.
    async fn animation(
        &self,
        tab: Option<u64>,
    ) -> Result<(String, lume_domain::anim::AnimResult), CallToolResult> {
        let doc = self.snapshot(tab).await?;
        let Some(anim) = doc.animation.clone() else {
            return Err(fail(
                "el documento no tiene animación de pixel art (anim.js): créala con write_animation",
            ));
        };
        let base = anim
            .base
            .clone()
            .unwrap_or_else(|| lume_domain::anim::Image::from_canvas(doc.canvas()));
        let loader: BaseLoader = Arc::new(move |_name: &str| Ok(base.clone()));
        let mut res = self.engines()?.anim.render(&anim.source, loader);
        lume_domain::anim::check::check(&mut res);
        Ok((anim.source, res))
    }

    async fn apply_animation(
        &self,
        tab: Option<u64>,
        source: String,
    ) -> Result<CallToolResult, ErrorData> {
        let studio = match self.studio() {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };
        if let Err(e) = studio.set_animation(tab, source).await {
            return Ok(fail(e.to_string()));
        }
        match self.animation(tab).await {
            Ok((_, res)) => Ok(ok(format!("anim.js guardado.\n{}", anim_report(&res)))),
            Err(e) => Ok(e),
        }
    }
}

fn anim_report(res: &lume_domain::anim::AnimResult) -> String {
    let mut out = format!("Celda {} × {}\n", res.cell.0, res.cell.1);
    for a in &res.actions {
        out.push_str(&format!(
            "- acción {}: {} fotogramas a {} fps, {}\n",
            a.spec.name,
            a.images.len(),
            a.spec.fps,
            if a.spec.looping {
                "en bucle"
            } else {
                "una vez"
            }
        ));
    }
    let (lines, _) = lume_domain::anim::check::issue_lines(res);
    if lines.is_empty() {
        out.push_str("Sin problemas.\n");
    } else {
        out.push_str(&format!("Problemas:\n{}\n", lines.join("\n")));
    }
    out
}

/// PNG del dibujo vectorial (entero o una zona, con rejilla si se pide).
fn render_png(
    engines: &Engines,
    res: &VecResult,
    width: u32,
    crop: Option<[f64; 4]>,
    grid: Option<f64>,
) -> Result<Vec<u8>, String> {
    let options = RasterOptions {
        width: Some(width.clamp(16, 4096)),
        crop,
        grid,
        ..Default::default()
    };
    let img = engines
        .rasterizer
        .rasterize(&res.clean, res.width as f64, res.height as f64, &options)
        .map_err(|e| e.to_string())?;
    encode_rgba(engines, img.width, img.height, &img.rgba)
}

/// PNG de unos píxeles RGBA.
fn encode_rgba(engines: &Engines, w: u32, h: u32, rgba: &[u8]) -> Result<Vec<u8>, String> {
    let pixels = rgba
        .chunks_exact(4)
        .map(|p| lume_domain::Rgba::new(p[0], p[1], p[2], p[3]))
        .collect();
    let image = lume_domain::anim::Image::from_pixels(w, h, pixels)
        .ok_or_else(|| "imagen de tamaño equivocado".to_string())?;
    engines.store.encode_image(&image).map_err(|e| e.message)
}

fn port(e: PortError) -> CallToolResult {
    fail(e.to_string())
}

// --- Herramientas ----------------------------------------------------------

#[tool_router(router = studio_router, vis = "pub(crate)")]
impl LumeMcp {
    #[tool(
        description = "El documento de una pestaña (tab: su id; sin él, la activa): nombre, archivo, tamaño, paleta, últimos pasos del historial, versiones, su dibujo vectorial (capas, problemas, esqueleto y acciones), su animación de pixel art y las pestañas de la ventana. Llámala para saber con qué trabajas."
    )]
    async fn document(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        let doc = match self.snapshot(a.tab).await {
            Ok(d) => d,
            Err(e) => return Ok(e),
        };
        let mut out = format!(
            "Documento «{}» · lienzo de píxeles {} × {}\n",
            doc.name,
            doc.canvas().width(),
            doc.canvas().height()
        );
        let palette = doc.palette();
        out.push_str(&format!("Paleta: {} colores\n", palette.len()));
        let entries = doc.history().entries();
        if !entries.is_empty() {
            out.push_str("Últimos pasos (el más nuevo al final):\n");
            for e in entries.iter().rev().take(10).rev() {
                out.push_str(&format!("- {}\n", e.label));
            }
        }
        if !doc.versions().is_empty() {
            out.push_str("Versiones:\n");
            for v in doc.versions() {
                out.push_str(&format!("- {}: {}\n", v.number, v.message));
            }
        }
        if let (Some(drawing), Ok(engines)) = (&doc.vector, self.engines()) {
            let res = engines.vector.render(&drawing.source);
            out.push_str(&format!(
                "\nDibujo vectorial (dibujo.js):\n{}",
                vector_report(&res)
            ));
        }
        if doc.animation.is_some() {
            match self.animation(a.tab).await {
                Ok((_, res)) => {
                    out.push_str(&format!(
                        "\nAnimación de pixel art (anim.js):\n{}",
                        anim_report(&res)
                    ));
                }
                Err(_) => out.push_str("\nTiene animación de pixel art (anim.js).\n"),
            }
        }
        if let Ok(studio) = self.studio()
            && let Ok(state) = studio.window(a.tab, WindowRequest::State).await
        {
            out.push_str(&format!("\nVentana:\n{}", window_report(&state)));
        }
        Ok(ok(out))
    }

    #[tool(description = "Deshace (o rehace, con redo: true) pasos del documento, como Ctrl+Z.")]
    async fn undo(&self, Parameters(a): Parameters<Steps>) -> Result<CallToolResult, ErrorData> {
        let studio = match self.studio() {
            Ok(s) => s,
            Err(e) => return Ok(e),
        };
        let redo = a.redo.unwrap_or(false);
        match studio.undo(a.tab, a.steps.unwrap_or(1).max(1), redo).await {
            Ok(n) => Ok(ok(format!(
                "{} {n} paso{}.",
                if redo { "Rehechos" } else { "Deshechos" },
                if n == 1 { "" } else { "s" }
            ))),
            Err(e) => Ok(port(e)),
        }
    }

    #[tool(
        description = "Guarda una versión del documento con un mensaje (se puede volver a ella con restore_version)."
    )]
    async fn save_version(
        &self,
        Parameters(a): Parameters<Message>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.studio() {
            Ok(s) => match s.save_version(a.tab, a.message).await {
                Ok(n) => Ok(ok(format!("Versión {n} guardada."))),
                Err(e) => Ok(port(e)),
            },
            Err(e) => Ok(e),
        }
    }

    #[tool(
        description = "Vuelve a una versión del documento (como un paso que se puede deshacer)."
    )]
    async fn restore_version(
        &self,
        Parameters(a): Parameters<Number>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.studio() {
            Ok(s) => match s.restore_version(a.tab, a.number).await {
                Ok(()) => Ok(ok(format!("Vuelto a la versión {}.", a.number))),
                Err(e) => Ok(port(e)),
            },
            Err(e) => Ok(e),
        }
    }

    // --- Vector -----------------------------------------------------------

    #[tool(
        description = "El código del dibujo vectorial (dibujo.js) con números de línea. La API de dibujo.js (formas, estilos, capas, esqueleto con bone/part/action) está en la skill de LumeArt; check te dice los errores con su línea."
    )]
    async fn read_vector(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.vector(a.tab).await {
            Ok((source, res)) => Ok(ok(format!(
                "{}\n\n{}",
                numbered(&source),
                vector_report(&res)
            ))),
            Err(e) => Ok(e),
        }
    }

    #[tool(
        description = "Pone el código entero de dibujo.js (lo crea si el documento no tenía). Se ve al momento en la ventana, queda como un paso que se puede deshacer y devuelve sus problemas con línea, su esqueleto y una vista previa."
    )]
    async fn write_vector(
        &self,
        Parameters(a): Parameters<Source>,
    ) -> Result<CallToolResult, ErrorData> {
        self.apply_vector(a.tab, a.source).await
    }

    #[tool(
        description = "Cambia un trozo de dibujo.js: old_text (tal cual, una sola vez) por new_text. Mejor que write_vector para cambios pequeños. Devuelve lo mismo que write_vector."
    )]
    async fn edit_vector(
        &self,
        Parameters(a): Parameters<EditSource>,
    ) -> Result<CallToolResult, ErrorData> {
        let (source, _) = match self.vector(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        match replace_once(&source, &a.old_text, &a.new_text) {
            Ok(next) => self.apply_vector(a.tab, next).await,
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "El dibujo vectorial como imagen PNG: entero o una zona (crop), con una rejilla de coordenadas si se pide (grid). Para revisar tu trabajo."
    )]
    async fn view_vector(
        &self,
        Parameters(a): Parameters<ViewVector>,
    ) -> Result<CallToolResult, ErrorData> {
        let (_, res) = match self.vector(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        let engines = match self.engines() {
            Ok(e) => e,
            Err(e) => return Ok(e),
        };
        match render_png(engines, &res, a.width.unwrap_or(1024), a.crop, a.grid) {
            Ok(png) => Ok(CallToolResult::success(vec![png_block(&png)])),
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "Las acciones del esqueleto del dibujo vectorial fotograma a fotograma (una fila por acción, la primera es el reposo), para revisar una animación."
    )]
    async fn view_vector_animation(
        &self,
        Parameters(a): Parameters<ViewRig>,
    ) -> Result<CallToolResult, ErrorData> {
        let (_, res) = match self.vector(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        let engines = match self.engines() {
            Ok(e) => e,
            Err(e) => return Ok(e),
        };
        let only = a.action.unwrap_or_default();
        let poses = match lume_domain::vec::rig::poses_svg(
            res.rig.as_ref(),
            &only,
            i64::from(a.frames.unwrap_or(8)),
            0,
        ) {
            Ok(p) => p,
            Err(e) => {
                return Ok(fail(format!(
                    "{e}. Para animar: pon las partes que se mueven en part(hueso, ...) con su bone(...) y crea action(...)"
                )));
            }
        };
        let options = RasterOptions {
            width: Some(poses.out_width.clamp(256, 2400)),
            ..Default::default()
        };
        match engines
            .rasterizer
            .rasterize(&poses.svg, poses.width, poses.height, &options)
            .map_err(|e| e.to_string())
            .and_then(|img| encode_rgba(engines, img.width, img.height, &img.rgba))
        {
            Ok(png) => Ok(CallToolResult::success(vec![png_block(&png)])),
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "Exporta el dibujo vectorial: .png (al ancho dado), .svg, .pdf o .ai (Illustrator), con sus capas."
    )]
    async fn export_vector(
        &self,
        Parameters(a): Parameters<ExportVector>,
    ) -> Result<CallToolResult, ErrorData> {
        let (_, res) = match self.vector(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        let engines = match self.engines() {
            Ok(e) => e,
            Err(e) => return Ok(e),
        };
        let path = self.path(&a.path);
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let result = if ext == "png" {
            render_png(engines, &res, a.width.unwrap_or(1024), None, None)
                .and_then(|png| std::fs::write(&path, png).map_err(|e| e.to_string()))
        } else {
            match &engines.exporter {
                Some(x) => x
                    .export(&res.clean, &path)
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                None => Err("no se puede exportar a ese formato aquí".into()),
            }
        };
        match result {
            Ok(()) => Ok(ok(format!("Exportado: {}", path.display()))),
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "Exporta una acción del esqueleto del dibujo vectorial como hoja de sprites (.png, fotogramas en fila) o GIF animado (.gif)."
    )]
    async fn export_vector_animation(
        &self,
        Parameters(a): Parameters<ExportRig>,
    ) -> Result<CallToolResult, ErrorData> {
        let (_, res) = match self.vector(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        let engines = match self.engines() {
            Ok(e) => e,
            Err(e) => return Ok(e),
        };
        let Some(rig) = res.rig.clone() else {
            return Ok(fail("el dibujo no tiene esqueleto"));
        };
        let Some(action) = rig.actions.iter().find(|x| x.name == a.action).cloned() else {
            let names: Vec<&str> = rig.actions.iter().map(|x| x.name.as_str()).collect();
            return Ok(fail(format!(
                "no hay acción {}: hay {}",
                a.action,
                names.join(", ")
            )));
        };
        let fps = a.fps.unwrap_or(12).clamp(1, 60);
        let width = a.width.unwrap_or(256).clamp(16, 2048);
        let n = ((action.duration * f64::from(fps)).round().max(1.0) as usize)
            + usize::from(!action.looping);
        let span = if action.looping { n } else { (n - 1).max(1) };
        let still = lume_domain::vec::studio::without_parts(&res.svg);
        let still = lume_domain::vec::studio::svg_inner(&still).to_string();
        let options = RasterOptions {
            width: Some(width),
            ..Default::default()
        };
        let mut frames = Vec::with_capacity(n);
        for i in 0..n {
            let t = action.duration * i as f64 / span as f64;
            let svg = rig.posed_svg(Some(&action), t, &still);
            match engines
                .rasterizer
                .rasterize(&svg, res.width as f64, res.height as f64, &options)
            {
                Ok(img) => frames.push(img),
                Err(e) => return Ok(fail(e.to_string())),
            }
        }
        let path = self.path(&a.path);
        let gif = path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("gif"));
        let to_image = |img: &lume_app::ports::RasterImage| {
            let pixels = img
                .rgba
                .chunks_exact(4)
                .map(|p| lume_domain::Rgba::new(p[0], p[1], p[2], p[3]))
                .collect();
            lume_domain::anim::Image::from_pixels(img.width, img.height, pixels)
        };
        let bytes: Result<Vec<u8>, String> = if gif {
            let images: Option<Vec<_>> = frames.iter().map(to_image).collect();
            let durations = vec![(1000 / fps).max(10); frames.len()];
            images
                .ok_or_else(|| "fotogramas de tamaños distintos".to_string())
                .and_then(|images| {
                    engines
                        .store
                        .encode_gif(&images, &durations, action.looping)
                        .map_err(|e| e.message)
                })
        } else {
            let (fw, fh) = (frames[0].width, frames[0].height);
            let total = fw * frames.len() as u32;
            let mut rgba = vec![0u8; (total * fh * 4) as usize];
            for (i, f) in frames.iter().enumerate() {
                for y in 0..fh.min(f.height) {
                    let src = (y * f.width * 4) as usize;
                    let dst = ((y * total + i as u32 * fw) * 4) as usize;
                    let len = (fw.min(f.width) * 4) as usize;
                    rgba[dst..dst + len].copy_from_slice(&f.rgba[src..src + len]);
                }
            }
            encode_rgba(engines, total, fh, &rgba)
        };
        match bytes.and_then(|b| std::fs::write(&path, b).map_err(|e| e.to_string())) {
            Ok(()) => Ok(ok(format!(
                "Exportado: {} ({n} fotogramas a {fps} fps)",
                path.display()
            ))),
            Err(e) => Ok(fail(e)),
        }
    }

    // --- Animación de pixel art -----------------------------------------------

    #[tool(
        description = "El código de la animación de pixel art (anim.js) con números de línea y sus acciones."
    )]
    async fn read_animation(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        match self.animation(a.tab).await {
            Ok((source, res)) => Ok(ok(format!(
                "{}\n\n{}",
                numbered(&source),
                anim_report(&res)
            ))),
            Err(e) => Ok(e),
        }
    }

    #[tool(
        description = "Pone el código entero de anim.js (la animación de pixel art; la base es el lienzo del documento). Devuelve sus acciones y problemas."
    )]
    async fn write_animation(
        &self,
        Parameters(a): Parameters<Source>,
    ) -> Result<CallToolResult, ErrorData> {
        self.apply_animation(a.tab, a.source).await
    }

    #[tool(
        description = "Cambia un trozo de anim.js: old_text (tal cual, una sola vez) por new_text."
    )]
    async fn edit_animation(
        &self,
        Parameters(a): Parameters<EditSource>,
    ) -> Result<CallToolResult, ErrorData> {
        let (source, _) = match self.animation(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        match replace_once(&source, &a.old_text, &a.new_text) {
            Ok(next) => self.apply_animation(a.tab, next).await,
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "La animación de pixel art como hoja: el reposo y una fila por acción, ampliada."
    )]
    async fn view_animation(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        let (_, res) = match self.animation(a.tab).await {
            Ok(v) => v,
            Err(e) => return Ok(e),
        };
        let engines = match self.engines() {
            Ok(e) => e,
            Err(e) => return Ok(e),
        };
        let side = res.cell.0.max(res.cell.1).max(1);
        let scale = (128 / side).clamp(1, 16);
        let sheet = lume_domain::anim::sheet::sheet(&res, scale);
        match engines.store.encode_image(&sheet) {
            Ok(png) => Ok(CallToolResult::success(vec![png_block(&png)])),
            Err(e) => Ok(fail(e.message)),
        }
    }

    // --- Ventana --------------------------------------------------------------

    #[tool(
        description = "Las pestañas abiertas en la ventana con su id (el `tab` de las demás herramientas), cuál es la activa, su modo, si tienen cambios, si su agente trabaja y si son un encargo de create_art. Cada pestaña es un documento independiente con su propio agente: puedes trabajar en varias a la vez pasando tab."
    )]
    async fn list_tabs(&self) -> Result<CallToolResult, ErrorData> {
        Ok(self.window(None, WindowRequest::State).await)
    }

    #[tool(
        description = "Documento nuevo en una pestaña nueva (devuelve su id: pásalo como tab al resto de herramientas para trabajar en ella sin cambiar la que ve el usuario): kind pixel (lienzo de píxeles de width × height), vector (dibujo.js de width × height unidades, 1024 por defecto) o animation."
    )]
    async fn new_document(
        &self,
        Parameters(a): Parameters<NewDoc>,
    ) -> Result<CallToolResult, ErrorData> {
        let doc = match kind_of(&a.kind) {
            Ok(k) => k,
            Err(e) => return Ok(fail(e)),
        };
        let default = if doc == DocKind::Vector { 1024 } else { 32 };
        Ok(self
            .window(
                None,
                WindowRequest::New {
                    doc,
                    width: a.width.unwrap_or(default),
                    height: a.height.unwrap_or(default),
                    palette: a.palette,
                },
            )
            .await)
    }

    #[tool(
        description = "Abre un .lume (o un PNG) en una pestaña nueva de la ventana (devuelve su id)."
    )]
    async fn open_document(
        &self,
        Parameters(a): Parameters<DocPath>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = self.path(&a.path);
        Ok(self.window(None, WindowRequest::Open { path }).await)
    }

    #[tool(
        description = "Guarda el documento de una pestaña (tab; sin él, la activa): en su archivo o en path (.lume)."
    )]
    async fn save_document(
        &self,
        Parameters(a): Parameters<MaybePath>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = a.path.map(|p| self.path(&p));
        Ok(self.window(a.tab, WindowRequest::Save { path }).await)
    }

    #[tool(
        description = "Enseña otra pestaña en la ventana (tab: su id). No hace falta para trabajar en ella: las demás herramientas aceptan tab."
    )]
    async fn select_tab(
        &self,
        Parameters(a): Parameters<TabIndex>,
    ) -> Result<CallToolResult, ErrorData> {
        if a.tab.is_none() && a.index.is_none() {
            return Ok(fail("di qué pestaña: tab (su id) o index"));
        }
        Ok(self
            .window(
                a.tab,
                WindowRequest::SelectTab {
                    index: a.index.unwrap_or(0),
                },
            )
            .await)
    }

    #[tool(
        description = "Cierra una pestaña de la ventana (si tiene cambios sin guardar, no: guárdala antes)."
    )]
    async fn close_tab(
        &self,
        Parameters(a): Parameters<TabIndex>,
    ) -> Result<CallToolResult, ErrorData> {
        if a.tab.is_none() && a.index.is_none() {
            return Ok(fail("di qué pestaña: tab (su id) o index"));
        }
        Ok(self
            .window(
                a.tab,
                WindowRequest::CloseTab {
                    index: a.index.unwrap_or(0),
                },
            )
            .await)
    }

    #[tool(
        description = "Cambia el nombre del documento de una pestaña (tab; sin él, la activa) y de su archivo, si lo tiene."
    )]
    async fn rename_document(
        &self,
        Parameters(a): Parameters<Name>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(self
            .window(a.tab, WindowRequest::Rename { name: a.name })
            .await)
    }

    #[tool(
        description = "Modo de una pestaña (tab; sin él, la activa): pixel, vector o animation (en un documento vectorial, animation es su esqueleto; en pixel art, anim.js)."
    )]
    async fn set_mode(
        &self,
        Parameters(a): Parameters<ModeArg>,
    ) -> Result<CallToolResult, ErrorData> {
        match kind_of(&a.mode) {
            Ok(mode) => Ok(self.window(a.tab, WindowRequest::SetMode { mode }).await),
            Err(e) => Ok(fail(e)),
        }
    }

    #[tool(
        description = "Le pide algo al agente de una pestaña (tab; sin él, la activa), como si lo escribiera el usuario en su panel («que la tapa se abra», «más contraste en el pelo»). Cada pestaña tiene su agente: puedes lanzar varios a la vez en pestañas distintas. No espera: sigue con window_agent_status (con el mismo tab)."
    )]
    async fn ask_window_agent(
        &self,
        Parameters(a): Parameters<Prompt>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(self
            .window(a.tab, WindowRequest::AskAgent { prompt: a.prompt })
            .await)
    }

    #[tool(
        description = "Si el agente de una pestaña (tab; sin él, la activa) sigue trabajando y lo último que dijo, y el estado de todas las pestañas."
    )]
    async fn window_agent_status(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(self.window(a.tab, WindowRequest::State).await)
    }

    #[tool(description = "Para al agente de una pestaña (tab; sin él, la activa).")]
    async fn stop_window_agent(
        &self,
        Parameters(a): Parameters<TabArg>,
    ) -> Result<CallToolResult, ErrorData> {
        Ok(self.window(a.tab, WindowRequest::StopAgent).await)
    }
}
