//! Herramientas del agente contra la `Session`: `draw`, `view_canvas`,
//! `get_pixel`, `undo`, `palette` y `write_notes`; con un motor de
//! animación, también `read_anim`, `write_anim` y `check_anim`; con un
//! motor de `dibujo.js`, `read_vec`, `write_vec` y `check_vec`.
//!
//! Todo lo que cambia el dibujo pasa por la sesión con
//! `Origin::Agent { run }`, así que queda en el historial como del agente.
//! Cada llamada a `draw` es un `batch`: un solo paso de deshacer.

use std::sync::Arc;

use lume_app::ports::AgentTools;
use lume_app::ports::{BoxFuture, ContentPart, ToolOutput, ToolSpec};
use lume_app::ports::{RasterOptions, Rasterizer};
use lume_app::{AnimEngine, DocumentStore, SessionHandle, VecEngine};
use lume_domain::anim::Image;
use lume_domain::anim::animation::AnimEdit;
use lume_domain::anim::check::{check, issue_lines, legend};
use lume_domain::anim::sheet::review;
use lume_domain::batch::split_batch;
use lume_domain::protocol::split_fields;
use lume_domain::vec::drawing::{VecEdit, changed_lines};
use lume_domain::vec::{VecResult, issue_lines as vec_issue_lines};
use lume_domain::{Origin, Palette, Rgba, RunId};
use serde_json::Value;

/// Nombres de las herramientas de pixel art, en el orden en que se ofrecen.
pub const TOOL_NAMES: [&str; 6] = [
    "draw",
    "view_canvas",
    "get_pixel",
    "undo",
    "palette",
    "write_notes",
];

/// Lado (en píxeles de pantalla) al que se amplía el lienzo en `view_canvas`.
const VIEW_SIDE: u32 = 512;

/// Herramientas de animación (con [`SessionTools::with_anim`]).
pub const ANIM_TOOL_NAMES: [&str; 3] = ["read_anim", "write_anim", "check_anim"];

/// Herramientas del dibujo vectorial (con [`SessionTools::with_vec`]).
pub const VEC_TOOL_NAMES: [&str; 3] = ["read_vec", "write_vec", "check_vec"];

/// Ancho de la imagen de `check_vec`.
const VEC_VIEW_WIDTH: u32 = 768;

/// Lo que necesitan las herramientas vectoriales: el motor de `dibujo.js` y
/// el rasterizador.
#[derive(Clone)]
pub struct VecTools {
    pub engine: Arc<dyn VecEngine>,
    pub rasterizer: Arc<dyn Rasterizer>,
}

/// Herramientas de una ejecución del agente sobre una sesión.
pub struct SessionTools {
    session: SessionHandle,
    store: Arc<dyn DocumentStore>,
    run: RunId,
    anim: Option<Arc<dyn AnimEngine>>,
    vector: Option<VecTools>,
}

impl SessionTools {
    /// `store` se usa para codificar el PNG de `view_canvas`.
    pub fn new(session: SessionHandle, store: Arc<dyn DocumentStore>, run: RunId) -> Self {
        Self {
            session,
            store,
            run,
            anim: None,
            vector: None,
        }
    }

    /// Añade las herramientas de animación (`read_anim`, `write_anim`,
    /// `check_anim`), que dibujan `anim.js` con este motor.
    pub fn with_anim(mut self, engine: Arc<dyn AnimEngine>) -> Self {
        self.anim = Some(engine);
        self
    }

    /// Añade las herramientas del dibujo vectorial (`read_vec`,
    /// `write_vec`, `check_vec`).
    pub fn with_vec(mut self, tools: VecTools) -> Self {
        self.vector = Some(tools);
        self
    }

    /// Ejecuta el `dibujo.js` del documento.
    async fn render_vec(&self) -> Result<VecResult, String> {
        let Some(tools) = self.vector.clone() else {
            return Err("no hay motor de dibujo vectorial".into());
        };
        let document = self.session.snapshot().await.map_err(|e| e.to_string())?;
        let Some(drawing) = document.vector.clone() else {
            return Err("este documento aún no tiene dibujo vectorial".into());
        };
        tokio::task::spawn_blocking(move || tools.engine.render(&drawing.source))
            .await
            .map_err(|e| e.to_string())
    }

    /// Tamaño, formas y problemas, como `lumeart vec check`.
    fn vec_report(result: &VecResult) -> String {
        let mut lines = vec![format!(
            "{}x{} · {} formas · SVG de {} KB",
            result.width,
            result.height,
            result.count,
            result.clean.len() / 1024
        )];
        lines.extend(vec_issue_lines(result));
        lines.join("\n")
    }

    async fn read_vec(&self) -> ToolOutput {
        match self.session.snapshot().await {
            Ok(document) => match document.vector {
                Some(drawing) => ToolOutput::text(drawing.source),
                None => ToolOutput::error("este documento aún no tiene dibujo vectorial"),
            },
            Err(err) => ToolOutput::error(format!("read_vec: {err}")),
        }
    }

    /// Cambia `dibujo.js` (un paso de deshacer) y guarda una versión con el
    /// cambio, como pxcli guardaba una copia antes de cada cambio del agente.
    async fn write_vec(&self, input: &Value) -> ToolOutput {
        let Some(source) = input.get("source").and_then(Value::as_str) else {
            return ToolOutput::error("write_vec necesita `source`: el dibujo.js completo");
        };
        let before = match self.session.snapshot().await {
            Ok(document) => document.vector.map(|d| d.source),
            Err(err) => return ToolOutput::error(format!("write_vec: {err}")),
        };
        match self
            .session
            .edit_vector(VecEdit::Source(source.to_string()), self.origin())
            .await
        {
            Ok(true) => {
                let lines = before
                    .as_deref()
                    .and_then(|b| changed_lines(b, source))
                    .map(|(a, b)| {
                        if a == b {
                            format!("línea {a}")
                        } else {
                            format!("líneas {a}–{b}")
                        }
                    })
                    .unwrap_or_else(|| "dibujo.js".into());
                let _ = self
                    .session
                    .create_version(self.origin(), format!("Agente · {lines}"))
                    .await;
            }
            Ok(false) => {}
            Err(err) => return ToolOutput::error(format!("write_vec: {err}")),
        }
        match self.render_vec().await {
            Ok(result) => {
                let mut out = ToolOutput::text(format!(
                    "ok: dibujo.js guardado.\n{}",
                    Self::vec_report(&result)
                ));
                out.is_error = !result.errors().is_empty();
                out
            }
            Err(err) => ToolOutput::error(format!("write_vec: {err}")),
        }
    }

    /// Los problemas y la imagen del dibujo (o de una zona, con su rejilla).
    async fn check_vec(&self, input: &Value) -> ToolOutput {
        let Some(tools) = self.vector.clone() else {
            return ToolOutput::error("check_vec: no hay motor de dibujo vectorial");
        };
        let result = match self.render_vec().await {
            Ok(result) => result,
            Err(err) => return ToolOutput::error(format!("check_vec: {err}")),
        };
        let mut out = ToolOutput::text(Self::vec_report(&result));
        if result.clean.is_empty() {
            return out;
        }
        let crop = input.get("crop").and_then(Value::as_array).and_then(|c| {
            let v: Vec<f64> = c.iter().filter_map(Value::as_f64).collect();
            (v.len() == 4 && v[2] > 0.0 && v[3] > 0.0).then(|| [v[0], v[1], v[2], v[3]])
        });
        let options = RasterOptions {
            width: Some(VEC_VIEW_WIDTH),
            crop,
            grid: input
                .get("grid")
                .and_then(Value::as_f64)
                .filter(|g| *g > 0.0),
            ..RasterOptions::default()
        };
        let (w, h) = (result.width as f64, result.height as f64);
        let image = tokio::task::spawn_blocking(move || {
            tools.rasterizer.rasterize(&result.clean, w, h, &options)
        })
        .await;
        match image {
            Ok(Ok(image)) => {
                let pixels = image
                    .rgba
                    .chunks_exact(4)
                    .map(|p| Rgba::new(p[0], p[1], p[2], p[3]))
                    .collect();
                if let Some(img) = Image::from_pixels(image.width, image.height, pixels)
                    && let Ok(png) = self.store.encode_image(&img)
                {
                    out.content.push(ContentPart::Image {
                        media_type: "image/png".into(),
                        data: png,
                    });
                }
            }
            Ok(Err(err)) => out = ToolOutput::error(format!("check_vec: {err}")),
            Err(err) => out = ToolOutput::error(format!("check_vec: {err}")),
        }
        out
    }

    /// Dibuja y comprueba la animación del documento.
    async fn render_anim(&self) -> Result<lume_domain::anim::AnimResult, String> {
        let Some(engine) = self.anim.clone() else {
            return Err("no hay motor de animación".into());
        };
        let document = self.session.snapshot().await.map_err(|e| e.to_string())?;
        let Some(animation) = document.animation.clone() else {
            return Err("este dibujo aún no tiene animación".into());
        };
        let base = animation
            .base
            .clone()
            .unwrap_or_else(|| Image::from_canvas(document.canvas()));
        tokio::task::spawn_blocking(move || {
            let loader: lume_app::BaseLoader = Arc::new(move |_name: &str| Ok(base.clone()));
            let mut result = engine.render(&animation.source, loader);
            check(&mut result);
            result
        })
        .await
        .map_err(|e| e.to_string())
    }

    /// Leyenda y problemas, como `lumeart anim check`.
    fn anim_report(result: &lume_domain::anim::AnimResult) -> (String, usize) {
        let (lines, errors) = issue_lines(result);
        (format!("{}\n{}", legend(result), lines.join("\n")), errors)
    }

    async fn read_anim(&self) -> ToolOutput {
        match self.session.snapshot().await {
            Ok(document) => match document.animation {
                Some(animation) => ToolOutput::text(animation.source),
                None => ToolOutput::error("este dibujo aún no tiene animación"),
            },
            Err(err) => ToolOutput::error(format!("read_anim: {err}")),
        }
    }

    async fn write_anim(&self, input: &Value) -> ToolOutput {
        let Some(source) = input.get("source").and_then(Value::as_str) else {
            return ToolOutput::error("write_anim necesita `source`: el anim.js completo");
        };
        if let Err(err) = self
            .session
            .edit_animation(AnimEdit::Source(source.to_string()), self.origin())
            .await
        {
            return ToolOutput::error(format!("write_anim: {err}"));
        }
        match self.render_anim().await {
            Ok(result) => {
                let (report, errors) = Self::anim_report(&result);
                let mut out = ToolOutput::text(format!("ok: anim.js guardado.\n{report}"));
                out.is_error = errors > 0;
                out
            }
            Err(err) => ToolOutput::error(format!("write_anim: {err}")),
        }
    }

    async fn check_anim(&self) -> ToolOutput {
        let result = match self.render_anim().await {
            Ok(result) => result,
            Err(err) => return ToolOutput::error(format!("check_anim: {err}")),
        };
        let (report, _) = Self::anim_report(&result);
        let longest = result.cell.0.max(result.cell.1).max(1);
        let scale = (VIEW_SIDE / 2 / longest).clamp(1, 16);
        let mut out = ToolOutput::text(report);
        if let Ok(png) = self.store.encode_image(&review(&result, scale)) {
            out.content.push(ContentPart::Image {
                media_type: "image/png".into(),
                data: png,
            });
        }
        out
    }

    fn origin(&self) -> Origin {
        Origin::Agent { run: self.run }
    }

    async fn draw(&self, input: &Value) -> ToolOutput {
        let Some(commands) = input.get("commands").and_then(Value::as_str) else {
            return ToolOutput::error(
                "draw necesita `commands`: comandos del protocolo, uno por línea",
            );
        };
        let lines = split_batch(commands);
        if lines.is_empty() {
            return ToolOutput::error("draw: no hay ningún comando");
        }
        // El agente no escribe archivos: exportar lo hace el usuario.
        if let Some(line) = lines.iter().find(|l| first_command(&l.text) == "export") {
            return ToolOutput::error(format!(
                "line {}: export no está disponible para el agente; el usuario exporta desde la ventana",
                line.num
            ));
        }
        match self.session.batch(commands, self.origin(), None).await {
            Ok(report) => {
                let mut out = ToolOutput::text(report.output().trim_end());
                // Si nada funcionó, que el modelo lo vea como error.
                out.is_error = report.ok == 0 && report.failed > 0;
                out
            }
            Err(err) => ToolOutput::error(format!("draw: {err}")),
        }
    }

    async fn view_canvas(&self) -> ToolOutput {
        let canvas = self.session.view().borrow().canvas.clone();
        let (w, h) = (canvas.width(), canvas.height());
        let scale = (VIEW_SIDE / w.max(h).max(1)).clamp(1, 32);
        match self.store.encode_png(&canvas, scale) {
            Ok(png) => ToolOutput {
                content: vec![
                    ContentPart::Text(format!(
                        "Lienzo de {w}×{h} píxeles, ampliado ×{scale} ({}×{} en la imagen). (0, 0) es la esquina de arriba a la izquierda.",
                        w * scale,
                        h * scale,
                    )),
                    ContentPart::Image {
                        media_type: "image/png".into(),
                        data: png,
                    },
                ],
                is_error: false,
            },
            Err(err) => ToolOutput::error(format!("view_canvas: {}", err.message)),
        }
    }

    async fn get_pixel(&self, input: &Value) -> ToolOutput {
        let coord = |name: &str| input.get(name).and_then(Value::as_i64);
        let (Some(x), Some(y)) = (coord("x"), coord("y")) else {
            return ToolOutput::error("get_pixel necesita `x` e `y` (enteros)");
        };
        self.line(&format!("get_pixel {x} {y}")).await
    }

    async fn undo(&self) -> ToolOutput {
        self.line("undo").await
    }

    async fn palette(&self) -> ToolOutput {
        let view = self.session.view().borrow().clone();
        let used = Palette::from_canvas(&view.canvas);
        let list = |palette: &Palette| {
            palette
                .colors()
                .iter()
                .map(|c| match &c.name {
                    Some(name) => format!("{} {name}", hex(c.color)),
                    None => hex(c.color),
                })
                .collect::<Vec<_>>()
                .join(", ")
        };
        let text = match &view.palette.name {
            Some(name) => {
                let outside = view.palette.colors_outside(&view.canvas);
                let mut text = format!(
                    "Paleta fija {name} ({} colores); usa solo estos: {}.",
                    view.palette.len(),
                    list(&view.palette)
                );
                if !outside.is_empty() {
                    let colors: Vec<String> = outside.into_iter().map(hex).collect();
                    text.push_str(&format!(
                        "\nColores del lienzo fuera de la paleta: {}.",
                        colors.join(", ")
                    ));
                }
                text
            }
            None if used.is_empty() => {
                "Sin paleta fija. El lienzo está vacío: puedes usar cualquier color.".to_string()
            }
            None => format!(
                "Sin paleta fija. Colores del lienzo ({}): {}.",
                used.len(),
                list(&used)
            ),
        };
        ToolOutput::text(text)
    }

    async fn write_notes(&self, input: &Value) -> ToolOutput {
        let Some(text) = input.get("text").and_then(Value::as_str) else {
            return ToolOutput::error("write_notes necesita `text`: las notas completas");
        };
        match self.session.set_notes(text.trim()).await {
            Ok(()) => ToolOutput::text("ok: notas guardadas"),
            Err(err) => ToolOutput::error(format!("write_notes: {err}")),
        }
    }

    /// Una línea del protocolo; la respuesta tal cual (`ok …`/`err …`).
    async fn line(&self, line: &str) -> ToolOutput {
        match self.session.line(line, self.origin(), None).await {
            Ok(result) => {
                let text = result.response.to_string();
                let mut out = ToolOutput::text(text.trim_end());
                out.is_error = text.starts_with("err");
                out
            }
            Err(err) => ToolOutput::error(format!("{line}: {err}")),
        }
    }
}

impl AgentTools for SessionTools {
    fn specs(&self) -> Vec<ToolSpec> {
        let mut all = specs();
        if self.anim.is_some() {
            all.extend(anim_specs());
        }
        if self.vector.is_some() {
            all.extend(vec_specs());
        }
        all
    }

    fn call<'a>(&'a self, name: &'a str, input_json: &'a str) -> BoxFuture<'a, ToolOutput> {
        Box::pin(async move {
            let input: Value = match serde_json::from_str(if input_json.trim().is_empty() {
                "{}"
            } else {
                input_json
            }) {
                Ok(value) => value,
                Err(err) => {
                    return ToolOutput::error(format!(
                        "{name}: la entrada no es JSON válido: {err}"
                    ));
                }
            };
            match name {
                "draw" => self.draw(&input).await,
                "view_canvas" => self.view_canvas().await,
                "get_pixel" => self.get_pixel(&input).await,
                "undo" => self.undo().await,
                "palette" => self.palette().await,
                "write_notes" => self.write_notes(&input).await,
                "read_anim" if self.anim.is_some() => self.read_anim().await,
                "write_anim" if self.anim.is_some() => self.write_anim(&input).await,
                "check_anim" if self.anim.is_some() => self.check_anim().await,
                "read_vec" if self.vector.is_some() => self.read_vec().await,
                "write_vec" if self.vector.is_some() => self.write_vec(&input).await,
                "check_vec" if self.vector.is_some() => self.check_vec(&input).await,
                other => ToolOutput::error(format!(
                    "no existe la herramienta `{other}`; usa {}",
                    TOOL_NAMES.join(", ")
                )),
            }
        })
    }
}

/// Descripción de las herramientas para el modelo.
pub fn specs() -> Vec<ToolSpec> {
    let spec = |name: &str, description: &str, schema: &str| ToolSpec {
        name: name.into(),
        description: description.into(),
        input_schema_json: schema.into(),
    };
    vec![
        spec(
            "draw",
            "Dibuja en el lienzo con comandos del protocolo de lumeart, uno por línea (o separados por `;`): `set_pixel X Y COLOR`, `fill_rect X Y W H COLOR`, `line X1 Y1 X2 Y2 COLOR`, `clear [COLOR]`. Los colores van como `#rrggbb`, `#rrggbbaa` o nombres (`red`). Cada llamada es un paso de deshacer. Devuelve `batch: N ok, M failed` y una línea por cada error.",
            r#"{"type":"object","properties":{"commands":{"type":"string","description":"Comandos del protocolo, uno por línea"}},"required":["commands"],"additionalProperties":false}"#,
        ),
        spec(
            "view_canvas",
            "Devuelve una imagen PNG ampliada del lienzo actual para revisar el dibujo.",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
        spec(
            "get_pixel",
            "Devuelve el color de un píxel (`ok #rrggbbaa`). (0, 0) es la esquina de arriba a la izquierda.",
            r#"{"type":"object","properties":{"x":{"type":"integer"},"y":{"type":"integer"}},"required":["x","y"],"additionalProperties":false}"#,
        ),
        spec(
            "undo",
            "Deshace el último paso (la última llamada a `draw`).",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
        spec(
            "palette",
            "Devuelve la paleta fija del dibujo (si la hay, usa solo esos colores) o los colores que ya tiene el lienzo.",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
        spec(
            "write_notes",
            "Guarda las notas del dibujo (notas.md): qué hiciste, qué falta y qué no funcionó. Sustituye a las anteriores, así que escríbelas enteras.",
            r#"{"type":"object","properties":{"text":{"type":"string","description":"Las notas completas, en Markdown"}},"required":["text"],"additionalProperties":false}"#,
        ),
    ]
}

/// Descripción de las herramientas de animación para el modelo.
pub fn anim_specs() -> Vec<ToolSpec> {
    let spec = |name: &str, description: &str, schema: &str| ToolSpec {
        name: name.into(),
        description: description.into(),
        input_schema_json: schema.into(),
    };
    vec![
        spec(
            "read_anim",
            "Devuelve el anim.js actual de la animación del dibujo.",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
        spec(
            "write_anim",
            "Sustituye el anim.js entero (un paso de deshacer) y devuelve las comprobaciones, como `lumeart anim check`. La API de anim.js: config({ cell, origin, colors }), part(nombre, (params) => {...}), rest(() => {...}), action(nombre, { frames, fps, loop }, (f, { t, n }) => {...}), draw(pieza, params), base(), move(dx, dy, fn), flip_x(fn), set_pixel, fill_rect, line, ellipse, clear, outline, lerp, ease, wave, pingpong, cycle, rand.",
            r#"{"type":"object","properties":{"source":{"type":"string","description":"El anim.js completo"}},"required":["source"],"additionalProperties":false}"#,
        ),
        spec(
            "check_anim",
            "Dibuja la animación y devuelve sus comprobaciones y la hoja de revisión (el reposo y una fila por acción) como imagen.",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
    ]
}

/// Descripción de las herramientas del dibujo vectorial para el modelo.
pub fn vec_specs() -> Vec<ToolSpec> {
    let spec = |name: &str, description: &str, schema: &str| ToolSpec {
        name: name.into(),
        description: description.into(),
        input_schema_json: schema.into(),
    };
    vec![
        spec(
            "read_vec",
            "Devuelve el dibujo.js actual del dibujo vectorial.",
            r#"{"type":"object","properties":{},"additionalProperties":false}"#,
        ),
        spec(
            "write_vec",
            "Sustituye el dibujo.js entero (un paso de deshacer y una versión nueva) y devuelve sus comprobaciones: tamaño, formas y errores con su línea.",
            r#"{"type":"object","properties":{"source":{"type":"string","description":"El dibujo.js completo"}},"required":["source"],"additionalProperties":false}"#,
        ),
        spec(
            "check_vec",
            "Ejecuta el dibujo.js y devuelve sus comprobaciones y su imagen. crop [x, y, ancho, alto] amplía una zona; grid N superpone una rejilla con coordenadas cada N unidades.",
            r#"{"type":"object","properties":{"crop":{"type":"array","items":{"type":"number"},"minItems":4,"maxItems":4},"grid":{"type":"number"}},"additionalProperties":false}"#,
        ),
    ]
}

/// Primer comando de una línea, sin `lumeart`/`pxcli` delante.
fn first_command(line: &str) -> String {
    let fields = split_fields(line, char::is_whitespace);
    let mut fields = fields
        .iter()
        .map(|f| f.trim_matches(|c| c == '"' || c == '\''));
    let first = fields.next().unwrap_or_default();
    let is_program = ["pxcli", "lumeart"]
        .iter()
        .any(|p| first == *p || first.ends_with(&format!("/{p}")));
    if is_program {
        fields.next().unwrap_or_default().to_string()
    } else {
        first.to_string()
    }
}

/// `#rrggbb` si es opaco; `#rrggbbaa` si no.
fn hex(c: Rgba) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        c.to_string()
    }
}
