//! Servidor MCP de LumeArt (`lumeart mcp`).
//!
//! Herramientas: `create_art`, `wait_art`, `revise_art`, `list_art`,
//! `art_guide` (también como recurso `lumeart://guia`) y, de bajo nivel,
//! `draw`, `view_canvas` y `export`. Ver `docs/03-arquitectura.md`, sección
//! «Uso desde otros agentes».
//!
//! Es una capa fina sobre dos puertos: [`ArtService`] (los encargos y el
//! lienzo, en este proceso o en el studio abierto) y [`ProjectStore`] (la
//! guía y los archivos del juego). Los problemas que debe ver el agente que
//! llama (un nombre repetido, una guía mal escrita) vuelven como resultado
//! con error, no como error del protocolo.
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use base64::Engine;
use lume_app::art::PREVIEW_SIDE;
use lume_app::{ArtId, ArtService, ArtSpec, ArtState, ArtStatus, PaletteSpec, ProjectStore};
use lume_domain::ArtGuide;
use lume_domain::guide::GuidePalette;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, ContentBlock, Implementation, ListResourcesResult, PaginatedRequestParams,
    ProgressNotificationParam, ReadResourceRequestParams, ReadResourceResponse, ReadResourceResult,
    RequestMetaObject, Resource, ResourceContents, ServerCapabilities, ServerConfig,
};
use rmcp::service::RequestContext;
use rmcp::{
    ErrorData, Peer, RoleServer, ServerHandler, ServiceExt, schemars, tool, tool_handler,
    tool_router,
};
use serde::Deserialize;

pub mod studio_tools;
pub use studio_tools::{Engines, STUDIO_TOOLS};

/// Nombres de las herramientas, en orden alfabético (la skill las nombra todas).
pub const TOOLS: [&str; 8] = [
    "art_guide",
    "create_art",
    "draw",
    "export",
    "list_art",
    "revise_art",
    "view_canvas",
    "wait_art",
];

/// URI del recurso con la guía de arte.
pub const GUIDE_URI: &str = "lumeart://guia";

/// Espera por defecto de `wait_art` y máxima.
pub const DEFAULT_WAIT: u64 = 30;
pub const MAX_WAIT: u64 = 300;

/// Cada cuánto mira `wait_art` el estado del encargo.
const POLL: Duration = Duration::from_millis(200);

/// Lado de la imagen de `view_canvas`.
const VIEW_SIDE: u32 = 512;

const INSTRUCTIONS: &str = "LumeArt dibuja el arte del juego con su propio agente y su bucle de refinado. \
Llama primero a `art_guide`. Encarga cada recurso con `create_art` (devuelve un número al momento), \
espera con `wait_art` (vuelve a llamarla mientras diga que sigue en marcha) y mira la vista previa. \
Si no encaja, `revise_art` con lo que falla en concreto. Los archivos quedan en la carpeta de la guía \
(`lumeart.toml`). `draw`, `view_canvas` y `export` son de bajo nivel: el lienzo de siempre con el \
protocolo de pxcli. Además puedes hacer todo lo de la ventana de LumeArt sobre el documento abierto \
(la pestaña activa): `document` para saber qué hay; `read_vector`, `edit_vector`, `write_vector`, \
`view_vector` y `export_vector` para el dibujo vectorial (dibujo.js); su esqueleto se anima con \
bone/part/action y se revisa con `view_vector_animation` y se exporta con `export_vector_animation`; \
`read_animation`, `write_animation` y `view_animation` para la animación de pixel art (anim.js); \
`undo`, `save_version` y `restore_version`; y la ventana: `list_tabs`, `new_document`, \
`open_document`, `save_document`, `select_tab`, `close_tab`, `rename_document`, `set_mode` y su \
agente (`ask_window_agent`, `window_agent_status`, `stop_window_agent`). Lo que cambias se ve al \
momento en la ventana y se puede deshacer. Cada pestaña es un documento independiente con su propio \
agente y un id fijo (lo dan `list_tabs`, `new_document` y `open_document`): todas estas herramientas \
aceptan `tab` para trabajar en esa pestaña sin cambiar la que ve el usuario, así que puedes llevar \
varias piezas a la vez. Los encargos de `create_art` también salen como pestañas.";

/// El servidor MCP.
#[derive(Clone)]
pub struct LumeMcp {
    art: Arc<dyn ArtService>,
    project: Arc<dyn ProjectStore>,
    /// Carpeta desde la que trabaja el agente que llama.
    cwd: PathBuf,
    /// El documento del studio y su ventana (las herramientas de `studio_tools`).
    studio: Option<Arc<dyn lume_app::studio::StudioAccess>>,
    /// Los motores para pintar y exportar.
    engines: Option<Engines>,
    tool_router: ToolRouter<Self>,
}

/// El proyecto del juego: su carpeta y su guía.
struct Project {
    dir: PathBuf,
    guide: ArtGuide,
    /// Ruta y texto de `lumeart.toml`, si hay.
    file: Option<(PathBuf, String)>,
}

impl Project {
    fn output(&self) -> PathBuf {
        self.dir.join(&self.guide.output)
    }
}

fn fail(message: impl Into<String>) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message.into())])
}

fn png_block(bytes: &[u8]) -> ContentBlock {
    ContentBlock::image(
        base64::engine::general_purpose::STANDARD.encode(bytes),
        "image/png",
    )
}

/// Ruta para mostrar: relativa al proyecto si está dentro.
fn shown(path: &Path, base: &Path) -> String {
    path.strip_prefix(base)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

/// Resumen de la guía para el agente.
fn describe(guide: &ArtGuide) -> String {
    let palette = match &guide.palette {
        GuidePalette::Free => "libre (sin límite de colores)".to_string(),
        GuidePalette::Fixed(name) => format!("fija, {name}"),
        GuidePalette::Custom(colors) => {
            let list: Vec<String> = colors.iter().map(|c| c.to_string()).collect();
            format!("propia, {} colores: {}", colors.len(), list.join(", "))
        }
    };
    let style = if guide.style.trim().is_empty() {
        "sin definir (pregúntale al usuario y añádelo a lumeart.toml)".to_string()
    } else {
        guide.style.clone()
    };
    format!(
        "- Estilo: {style}\n- Tamaño por defecto: {}x{}\n- Paleta: {palette}\n- Carpeta de los archivos: {}\n- Nombres de archivo: {}\n",
        guide.size.0,
        guide.size.1,
        guide.output,
        guide.naming.as_str()
    )
}

fn status_json(status: &ArtStatus) -> serde_json::Value {
    serde_json::json!({
        "id": status.id,
        "name": status.name,
        "state": status.state.as_str(),
        "turn": status.turn,
        "max_turns": status.max_turns,
        "message": status.message,
        "png": status.png,
        "lume": status.lume,
        "written": status.written,
        "versions": status.versions,
    })
}

/// `create_art`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateArt {
    /// Qué es y para qué sirve en el juego («cofre de madera que el jugador abre»).
    pub description: String,
    /// Nombre del archivo, sin extensión («cofre_madera»); se adapta a la convención de la guía.
    pub name: String,
    /// Tipo de recurso: `sprite` (por defecto). `animation` y `vector` llegarán más adelante.
    #[serde(default)]
    pub kind: Option<String>,
    /// Tamaño ANCHOxALTO en píxeles («32x32»); por defecto, el de la guía.
    #[serde(default)]
    pub size: Option<String>,
    /// Vista: frontal, lateral, cenital, isométrica…
    #[serde(default)]
    pub view: Option<String>,
    /// Indicaciones de estilo para este recurso; se suman al estilo de la guía.
    #[serde(default)]
    pub style: Option<String>,
}

/// `wait_art`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WaitArt {
    /// Número del encargo que devolvió `create_art` o `revise_art`.
    pub id: ArtId,
    /// Segundos como mucho que espera esta llamada (por defecto 30, máximo 300).
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
}

/// `revise_art`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ReviseArt {
    /// Qué hay que cambiar, en concreto («el contorno es demasiado grueso»).
    pub changes: String,
    /// Número de un encargo de esta sesión.
    #[serde(default)]
    pub id: Option<ArtId>,
    /// O el nombre del recurso en la carpeta de la guía.
    #[serde(default)]
    pub name: Option<String>,
}

/// `draw`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Draw {
    /// Comandos del protocolo, uno por línea o separados por `;` (`set_pixel 1 1 red`, `fill_rect 0 0 4 4 #8b3fe0`…). Se deshacen de una vez.
    pub commands: String,
}

/// `export`.
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct Export {
    /// Archivo PNG de salida (relativo a la carpeta del agente).
    pub path: String,
    /// Amplía cada píxel a N × N (1–64).
    #[serde(default)]
    pub scale: Option<u32>,
}

impl LumeMcp {
    pub fn new(art: Arc<dyn ArtService>, project: Arc<dyn ProjectStore>, cwd: PathBuf) -> Self {
        Self {
            art,
            project,
            cwd,
            studio: None,
            engines: None,
            tool_router: Self::tool_router() + Self::studio_router(),
        }
    }

    /// Con acceso al documento del studio y los motores para pintar.
    pub fn with_studio(
        mut self,
        studio: Arc<dyn lume_app::studio::StudioAccess>,
        engines: Engines,
    ) -> Self {
        self.studio = Some(studio);
        self.engines = Some(engines);
        self
    }

    /// Sirve por la entrada y la salida estándar hasta que el cliente se va.
    pub async fn serve_stdio(self) -> Result<(), String> {
        let running = self
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| e.to_string())?;
        running.waiting().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    fn project(&self) -> Result<Project, String> {
        match self.project.find_guide(&self.cwd) {
            Ok(Some(file)) => Ok(Project {
                dir: file.project().to_path_buf(),
                guide: file.guide.clone(),
                file: Some((file.path.clone(), file.text.clone())),
            }),
            Ok(None) => Ok(Project {
                dir: self.cwd.clone(),
                guide: ArtGuide::default(),
                file: None,
            }),
            Err(e) => Err(format!("{e}\nCorrige lumeart.toml y vuelve a intentarlo.")),
        }
    }

    fn guide_text(&self) -> Result<String, String> {
        let project = self.project()?;
        let mut text = match &project.file {
            Some((path, _)) => format!("Guía de arte del proyecto ({}):\n", path.display()),
            None => format!(
                "Este proyecto aún no tiene lumeart.toml (se buscó desde {}). Se usan los valores por defecto; créalo con `lumeart init` en la carpeta del juego después de preguntar al usuario por el estilo y la paleta.\n",
                self.cwd.display()
            ),
        };
        text.push_str(&describe(&project.guide));
        text.push_str(&format!(
            "- Los archivos van a: {}\n",
            project.output().display()
        ));
        if let Some((_, raw)) = &project.file {
            text.push_str(&format!("\nlumeart.toml:\n```toml\n{raw}```\n"));
        }
        Ok(text)
    }

    /// Texto y bloques de un estado (con su vista previa si se puede).
    async fn report(&self, status: &ArtStatus, base: &Path) -> Vec<ContentBlock> {
        let mut text = format!(
            "Encargo {} · {}: {}.\n",
            status.id,
            status.name,
            status.state.label()
        );
        if status.state == ArtState::Running {
            if status.turn == 0 {
                text.push_str("Está empezando.");
            } else {
                text.push_str(&format!("Vuelta {} de {}.", status.turn, status.max_turns));
            }
            text.push_str(&format!(
                " Vuelve a llamar a wait_art con id {} para seguir esperando.\n",
                status.id
            ));
        } else {
            text.push_str(&format!("{}\n", status.message));
        }
        if status.written {
            text.push_str(&format!(
                "Archivos:\n- {}\n- {}\n",
                shown(&status.png, base),
                shown(&status.lume, base)
            ));
        }
        let mut blocks = vec![
            ContentBlock::text(text),
            ContentBlock::text(status_json(status).to_string()),
        ];
        if let Ok(png) = self.art.preview(status.id, PREVIEW_SIDE as u32).await {
            blocks.push(png_block(&png));
        }
        blocks
    }

    fn resolve_size(&self, guide: &ArtGuide, size: Option<&str>) -> Result<(i64, i64), String> {
        match size {
            Some(size) => ArtGuide::parse_size(size).map_err(|e| e.message),
            None => Ok(guide.size),
        }
    }
}

#[tool_router]
impl LumeMcp {
    /// Guía de arte del proyecto.
    #[tool(
        description = "Guía de arte del juego (lumeart.toml): estilo, tamaño, paleta, carpeta de salida y nombres. Llámala antes de encargar nada."
    )]
    async fn art_guide(&self) -> Result<CallToolResult, ErrorData> {
        Ok(match self.guide_text() {
            Ok(text) => CallToolResult::success(vec![ContentBlock::text(text)]),
            Err(e) => fail(e),
        })
    }

    #[tool(
        description = "Encarga un recurso de arte (un sprite). No espera: devuelve el número del encargo al momento; después llama a wait_art. El PNG y el .lume quedan en la carpeta de la guía."
    )]
    async fn create_art(
        &self,
        Parameters(args): Parameters<CreateArt>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = match self.project() {
            Ok(project) => project,
            Err(e) => return Ok(fail(e)),
        };
        match args.kind.as_deref().map(str::trim).unwrap_or("sprite") {
            "" | "sprite" => {}
            "animation" | "vector" => {
                return Ok(fail(format!(
                    "LumeArt todavía no hace encargos de tipo {:?}; de momento pide un sprite",
                    args.kind.unwrap_or_default()
                )));
            }
            other => {
                return Ok(fail(format!(
                    "tipo {other:?} desconocido; usa sprite, animation o vector"
                )));
            }
        }
        if args.description.trim().is_empty() {
            return Ok(fail(
                "falta la descripción: qué es y para qué sirve en el juego",
            ));
        }
        let Some(name) = project.guide.naming.apply(&args.name) else {
            return Ok(fail(format!(
                "el nombre {:?} no tiene letras ni cifras",
                args.name
            )));
        };
        let size = match self.resolve_size(&project.guide, args.size.as_deref()) {
            Ok(size) => size,
            Err(e) => return Ok(fail(format!("size: {e}"))),
        };
        let output = project.output();
        let png = output.join(format!("{name}.png"));
        let lume = output.join(format!("{name}.lume"));
        if let Some(existing) = [&png, &lume].into_iter().find(|p| p.exists()) {
            return Ok(fail(format!(
                "ya existe «{name}» ({}); usa revise_art para cambiarlo o elige otro nombre",
                shown(existing, &project.dir)
            )));
        }
        if let Err(e) = self.project.prepare_output(&output) {
            return Ok(fail(e.to_string()));
        }
        let style = [
            project.guide.style.trim(),
            args.style.as_deref().unwrap_or("").trim(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(". ");
        let spec = ArtSpec {
            name: name.clone(),
            description: args.description.trim().to_string(),
            size,
            view: args.view.unwrap_or_default().trim().to_string(),
            style,
            palette: PaletteSpec::of_guide(&project.guide),
            png,
            lume,
            base: None,
            changes: String::new(),
        };
        match self.art.create(spec).await {
            Ok(status) => Ok(CallToolResult::success(vec![
                ContentBlock::text(format!(
                    "Encargo {id} en marcha: «{name}» ({w}x{h}). Llama a wait_art con id {id}; puede tardar varios minutos.",
                    id = status.id,
                    w = size.0,
                    h = size.1
                )),
                ContentBlock::text(status_json(&status).to_string()),
            ])),
            Err(e) => Ok(fail(e.to_string())),
        }
    }

    #[tool(
        description = "Espera a un encargo como mucho timeout_seconds (30 por defecto) y devuelve su estado (running, done, failed, cancelled), la vuelta, las rutas de los archivos y una vista previa. Si sigue en marcha, vuelve a llamarla."
    )]
    async fn wait_art(
        &self,
        Parameters(args): Parameters<WaitArt>,
        meta: RequestMetaObject,
        client: Peer<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let wait = Duration::from_secs(
            args.timeout_seconds
                .unwrap_or(DEFAULT_WAIT)
                .clamp(1, MAX_WAIT),
        );
        let deadline = Instant::now() + wait;
        let token = meta.get_progress_token();
        let mut last_turn = None;
        let status = loop {
            let status = match self.art.status(args.id).await {
                Ok(status) => status,
                Err(e) => return Ok(fail(e.to_string())),
            };
            if let Some(token) = &token
                && last_turn != Some(status.turn)
            {
                last_turn = Some(status.turn);
                let _ = client
                    .notify_progress(
                        ProgressNotificationParam::new(token.clone(), f64::from(status.turn))
                            .with_total(f64::from(status.max_turns))
                            .with_message(format!(
                                "{}: vuelta {} de {}",
                                status.name, status.turn, status.max_turns
                            )),
                    )
                    .await;
            }
            if status.state != ArtState::Running || Instant::now() >= deadline {
                break status;
            }
            tokio::time::sleep(POLL).await;
        };
        let base = self
            .project()
            .map(|p| p.dir)
            .unwrap_or_else(|_| self.cwd.clone());
        let blocks = self.report(&status, &base).await;
        Ok(if status.state == ArtState::Failed {
            CallToolResult::error(blocks)
        } else {
            CallToolResult::success(blocks)
        })
    }

    #[tool(
        description = "Pide cambios sobre un recurso (por su número de encargo o por su nombre): LumeArt sigue el mismo dibujo y guarda una versión nueva. Devuelve un número de encargo nuevo para wait_art."
    )]
    async fn revise_art(
        &self,
        Parameters(args): Parameters<ReviseArt>,
    ) -> Result<CallToolResult, ErrorData> {
        let project = match self.project() {
            Ok(project) => project,
            Err(e) => return Ok(fail(e)),
        };
        if args.changes.trim().is_empty() {
            return Ok(fail("falta qué hay que cambiar (changes)"));
        }
        let (name, description, png, lume) = match (args.id, args.name.as_deref()) {
            (Some(id), _) => match self.art.status(id).await {
                Ok(status) if status.state == ArtState::Running => {
                    return Ok(fail(format!(
                        "el encargo {id} sigue en marcha; espera a que termine con wait_art"
                    )));
                }
                Ok(status) => (status.name, status.description, status.png, status.lume),
                Err(e) => return Ok(fail(e.to_string())),
            },
            (None, Some(name)) => {
                let Some(name) = project.guide.naming.apply(name) else {
                    return Ok(fail(format!(
                        "el nombre {name:?} no tiene letras ni cifras"
                    )));
                };
                let output = project.output();
                let png = output.join(format!("{name}.png"));
                let lume = output.join(format!("{name}.lume"));
                (name, String::new(), png, lume)
            }
            (None, None) => return Ok(fail("di qué recurso cambiar: id o name")),
        };
        if !lume.exists() {
            return Ok(fail(format!(
                "no hay ningún recurso «{name}» en {}; para uno nuevo usa create_art",
                shown(lume.parent().unwrap_or(Path::new(".")), &project.dir)
            )));
        }
        let spec = ArtSpec {
            name: name.clone(),
            description,
            size: project.guide.size,
            view: String::new(),
            style: project.guide.style.clone(),
            palette: PaletteSpec::of_guide(&project.guide),
            png,
            lume: lume.clone(),
            base: Some(lume),
            changes: args.changes.trim().to_string(),
        };
        match self.art.create(spec).await {
            Ok(status) => Ok(CallToolResult::success(vec![
                ContentBlock::text(format!(
                    "Revisión de «{name}» en marcha: encargo {id}. Llama a wait_art con id {id}.",
                    id = status.id
                )),
                ContentBlock::text(status_json(&status).to_string()),
            ])),
            Err(e) => Ok(fail(e.to_string())),
        }
    }

    #[tool(
        description = "Recursos del proyecto (los archivos de la carpeta de la guía) con su versión, y los encargos de esta sesión con su estado."
    )]
    async fn list_art(&self) -> Result<CallToolResult, ErrorData> {
        let project = match self.project() {
            Ok(project) => project,
            Err(e) => return Ok(fail(e)),
        };
        let output = project.output();
        let assets = match self.project.list_assets(&output) {
            Ok(assets) => assets,
            Err(e) => return Ok(fail(e.to_string())),
        };
        let jobs = self.art.list().await.unwrap_or_default();
        let mut text = format!("Recursos en {}:\n", shown(&output, &project.dir));
        if assets.is_empty() {
            text.push_str("(ninguno todavía)\n");
        }
        let mut items = Vec::new();
        for asset in &assets {
            let versions = match asset.versions {
                Some(1) => ", 1 versión".to_string(),
                Some(n) => format!(", {n} versiones"),
                None => String::new(),
            };
            let files: Vec<String> = [&asset.png, &asset.lume]
                .into_iter()
                .flatten()
                .map(|p| shown(p, &project.dir))
                .collect();
            text.push_str(&format!(
                "- {}{versions}: {}\n",
                asset.name,
                files.join(", ")
            ));
            items.push(serde_json::json!({
                "name": asset.name,
                "png": asset.png,
                "lume": asset.lume,
                "versions": asset.versions,
            }));
        }
        if !jobs.is_empty() {
            text.push_str("\nEncargos de esta sesión:\n");
            for job in &jobs {
                let turn = if job.state == ArtState::Running {
                    format!(", vuelta {} de {}", job.turn, job.max_turns)
                } else {
                    String::new()
                };
                text.push_str(&format!(
                    "- {} · {}: {}{turn}\n",
                    job.id,
                    job.name,
                    job.state.label()
                ));
            }
        }
        let json = serde_json::json!({
            "assets": items,
            "jobs": jobs.iter().map(status_json).collect::<Vec<_>>(),
        });
        Ok(CallToolResult::success(vec![
            ContentBlock::text(text),
            ContentBlock::text(json.to_string()),
        ]))
    }

    #[tool(
        description = "Bajo nivel: ejecuta comandos del protocolo de pxcli en el lienzo (set_pixel, fill_rect, line, clear, get_pixel, new, undo, redo…), uno por línea. Devuelve una respuesta por línea y el resumen."
    )]
    async fn draw(&self, Parameters(args): Parameters<Draw>) -> Result<CallToolResult, ErrorData> {
        match self.art.draw(args.commands, Some(self.cwd.clone())).await {
            Ok(output) => Ok(CallToolResult::success(vec![ContentBlock::text(output)])),
            Err(e) => Ok(fail(e.to_string())),
        }
    }

    #[tool(description = "Bajo nivel: el lienzo actual como imagen PNG ampliada.")]
    async fn view_canvas(&self) -> Result<CallToolResult, ErrorData> {
        match self.art.canvas_png(VIEW_SIDE).await {
            Ok(png) => Ok(CallToolResult::success(vec![png_block(&png)])),
            Err(e) => Ok(fail(e.to_string())),
        }
    }

    #[tool(description = "Bajo nivel: exporta el lienzo actual a un PNG (como `lumeart export`).")]
    async fn export(
        &self,
        Parameters(args): Parameters<Export>,
    ) -> Result<CallToolResult, ErrorData> {
        let path = self.cwd.join(&args.path);
        let mut line = format!(
            "export {}",
            lume_domain::protocol::quote_field(&path.to_string_lossy())
        );
        // Dentro de un `batch`, la escala va con `--scale`.
        if let Some(scale) = args.scale {
            line.push_str(&format!(" --scale {scale}"));
        }
        match self.art.draw(line, Some(self.cwd.clone())).await {
            Ok(output) if !output.contains(", 0 failed") => Ok(fail(output)),
            Ok(output) => Ok(CallToolResult::success(vec![ContentBlock::text(output)])),
            Err(e) => Ok(fail(e.to_string())),
        }
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for LumeMcp {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("lumeart", env!("CARGO_PKG_VERSION")))
        .with_instructions(INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(vec![
            Resource::new(GUIDE_URI, "art_guide")
                .with_title("Guía de arte")
                .with_description(
                    "lumeart.toml del proyecto: estilo, tamaño, paleta, carpeta y nombres",
                )
                .with_mime_type("text/markdown"),
        ]))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, ErrorData> {
        if request.uri != GUIDE_URI {
            return Err(ErrorData::resource_not_found(
                format!("no hay ningún recurso {}", request.uri),
                None,
            ));
        }
        let text = self
            .guide_text()
            .map_err(|e| ErrorData::internal_error(e, None))?;
        Ok(ReadResourceResult::new(vec![ResourceContents::text(text, GUIDE_URI)]).into())
    }
}
