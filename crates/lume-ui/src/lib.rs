//! Interfaz nativa de LumeArt (eframe/egui).
//!
//! Patrón Model–View–Update: la UI lee un `ViewModel` y emite `Intent`s; nunca
//! modifica el documento directamente. El armazón (`AppShell`) es igual en los
//! tres modos; cada modo implementa `StudioMode` y cada herramienta `Tool`.
//! El diseño de referencia está en `docs/04-diseno-ui.md` y `docs/diseno/`.
//!
//! Regla: depende de `lume-app` y `lume-domain`, nunca de otro adaptador.

pub mod agent_panel;
pub mod agent_run;
pub mod anim_player;
pub mod brush;
pub mod canvas_view;
pub mod dialogs;
pub mod home;
pub mod icons;
pub mod intent;
pub mod jobs;
pub mod markdown;
pub mod panel_tabs;
pub mod selection;
pub mod settings_screen;
pub mod shell;
mod size_picker;
pub mod syntax;
pub mod theme;
pub mod tools;
pub mod vec_draw;

use std::sync::Arc;

use eframe::egui;
use lume_app::ports::CancellationToken;
use lume_app::testing::{MemorySecretStore, MemorySettingsStore};
use lume_app::{
    AgentBackend, AnimEngine, ArtId, DocumentStore, JobEntry, LocalArt, ProviderKind, RecentEntry,
    RecentStore, SecretStore, SessionHandle, Settings, SettingsStore,
};
use lume_domain::anim::Image;
use lume_domain::anim::animation::{AnimEdit, Animation};
use lume_domain::anim::project::{ProjectOptions, new_project};
use lume_domain::vec::drawing::{VecEdit, VectorDrawing};
use lume_domain::{Document, Origin};

pub use dialogs::{FakeDialogs, FileDialogs, NativeDialogs};
pub use home::HomeScreen;
pub use intent::{Intent, Mode};
pub use settings_screen::{KeyStatus, Section, SettingsScreen};
pub use shell::AnimMode;
pub use shell::{AppShell, Tool, Zones};
pub use shell::{VecMode, VecTool, VecView, ZoneState};

/// Revisión que se da al `ViewModel` mientras se ve una versión (lejos de
/// las reales, para que el lienzo rehaga la textura al entrar y al salir).
const PREVIEW_REVISION: u64 = 1 << 62;

/// Título de la ventana.
pub const WINDOW_TITLE: &str = "LumeArt";

/// Tamaño máximo de una imagen de referencia.
const MAX_REFERENCE_BYTES: usize = 8 * 1024 * 1024;

/// PNG RGBA de 8 bits a partir de los píxeles (sin premultiplicar).
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(rgba).ok()?;
    writer.finish().ok()?;
    Some(out)
}

/// Decodifica una imagen (PNG, JPEG, WebP o GIF) a RGBA de 8 bits.
pub fn decode_image(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    if let Some(png) = decode_png(bytes) {
        return Some(png);
    }
    let image = image::load_from_memory(bytes).ok()?.to_rgba8();
    Some((image.width(), image.height(), image.into_raw()))
}

/// Decodifica un PNG a RGBA de 8 bits: `(ancho, alto, bytes)`.
pub fn decode_png(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let mut decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()?];
    let info = reader.next_frame(&mut buf).ok()?;
    let buf = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf.to_vec(),
        png::ColorType::Rgb => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    Some((info.width, info.height, rgba))
}

/// Lado del icono de la ventana. En Linux (X11) va en una sola petición al
/// servidor gráfico, que sin BIG-REQUESTS no pasa de 256 KB: el de 256×256
/// (262 KB) se perdía y la ventana quedaba sin icono. En Windows y macOS
/// eframe lo usa para la barra de tareas y el Dock, y ahí cuanto mayor, mejor.
pub const WINDOW_ICON_SIZE: u32 = if cfg!(target_os = "linux") { 128 } else { 256 };

/// Icono de la ventana (`assets/brand/icon-128.png` en Linux,
/// `icon-256.png` en los demás; embebido).
pub fn window_icon() -> egui::IconData {
    #[cfg(target_os = "linux")]
    let bytes = include_bytes!("../../../assets/brand/icon-128.png");
    #[cfg(not(target_os = "linux"))]
    let bytes = include_bytes!("../../../assets/brand/icon-256.png");
    let (width, height, rgba) = decode_png(bytes).expect("el icono embebido es un PNG");
    egui::IconData {
        rgba,
        width,
        height,
    }
}

/// El icono de las ventanas (se decodifica una vez).
fn shared_icon() -> Arc<egui::IconData> {
    static ICON: std::sync::OnceLock<Arc<egui::IconData>> = std::sync::OnceLock::new();
    ICON.get_or_init(|| Arc::new(window_icon())).clone()
}

/// Si la ventana de Ajustes ya estaba abierta, la trae delante.
fn ctx_focus_settings(ctx: &egui::Context) {
    ctx.send_viewport_cmd_to(
        egui::ViewportId::from_hash_of("lumeart-ajustes"),
        egui::ViewportCommand::Focus,
    );
}

/// Opciones de la ventana: título, tamaño de referencia e icono.
pub fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_app_id("lumeart")
            .with_inner_size(theme::WINDOW_SIZE)
            .with_min_inner_size(theme::WINDOW_MIN_SIZE)
            .with_icon(shared_icon()),
        ..Default::default()
    }
}

/// Pide repintar la ventana cada vez que cambia el documento (lo cambie la
/// ventana, la CLI, la API o el agente) y cuando piden parar. Sin cambios no
/// se repinta: la ventana no gasta CPU en reposo.
pub fn repaint_on_changes(
    runtime: &tokio::runtime::Handle,
    session: &SessionHandle,
    ctx: egui::Context,
    stop: CancellationToken,
) {
    let mut view = session.view();
    runtime.spawn(async move {
        loop {
            tokio::select! {
                changed = view.changed() => {
                    if changed.is_err() {
                        break;
                    }
                }
                _ = stop.cancelled() => {
                    ctx.request_repaint();
                    break;
                }
            }
            ctx.request_repaint();
        }
    });
}

/// Pantalla que muestra la ventana.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Screen {
    /// Inicio: nuevo dibujo y recientes.
    Home,
    /// El studio con el documento abierto.
    #[default]
    Studio,
    /// Ajustes.
    Settings,
}

/// Lleva a cabo los intents: los del documento van a la sesión; los de
/// archivo usan los puertos y los diálogos. Recuerda el archivo abierto y la
/// revisión guardada (para saber si hay cambios sin guardar).
pub struct AppController {
    pub session: SessionHandle,
    pub runtime: tokio::runtime::Handle,
    pub store: Arc<dyn DocumentStore>,
    pub recents: Option<Arc<dyn RecentStore>>,
    pub dialogs: Arc<dyn FileDialogs>,
    /// Archivo `.lume` del documento abierto (si se abrió o guardó).
    pub path: Option<std::path::PathBuf>,
    /// Revisión del documento cuando se guardó o abrió por última vez.
    pub saved_revision: Option<u64>,
    /// Ajustes actuales (los guarda `settings_store`).
    pub settings: Settings,
    pub settings_store: Arc<dyn SettingsStore>,
    /// Llavero: las claves de API solo pasan por aquí.
    pub secrets: Arc<dyn SecretStore>,
    /// Qué claves hay (enmascaradas); se lee al abrir Ajustes.
    pub keys: settings_screen::KeyStatus,
}

/// Resultado de un intent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Outcome {
    pub screen: Option<Screen>,
    pub error: Option<String>,
    /// Mensaje informativo («Guardado en …»).
    pub notice: Option<String>,
}

impl Outcome {
    fn error(message: impl Into<String>) -> Self {
        Self {
            error: Some(message.into()),
            ..Self::default()
        }
    }

    fn notice(message: impl Into<String>) -> Self {
        Self {
            notice: Some(message.into()),
            ..Self::default()
        }
    }

    fn studio() -> Self {
        Self {
            screen: Some(Screen::Studio),
            ..Self::default()
        }
    }
}

/// Segundos desde 1970.
pub fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl AppController {
    pub fn new(
        session: SessionHandle,
        runtime: tokio::runtime::Handle,
        store: Arc<dyn DocumentStore>,
        recents: Option<Arc<dyn RecentStore>>,
        dialogs: Arc<dyn FileDialogs>,
    ) -> Self {
        let saved_revision = Some(session.view().borrow().revision);
        Self {
            session,
            runtime,
            store,
            recents,
            dialogs,
            path: None,
            saved_revision,
            settings: Settings::default(),
            settings_store: Arc::new(MemorySettingsStore::default()),
            secrets: Arc::new(MemorySecretStore::default()),
            keys: settings_screen::KeyStatus::default(),
        }
    }

    /// Usa estos ajustes y este llavero (sin llamarlo, los ajustes viven solo
    /// en memoria). Si los ajustes guardados no se pueden leer, se usan los de
    /// fábrica y se devuelve el motivo.
    pub fn with_settings(
        mut self,
        store: Arc<dyn SettingsStore>,
        secrets: Arc<dyn SecretStore>,
    ) -> (Self, Option<String>) {
        let problem = match store.load() {
            Ok(settings) => {
                self.settings = settings;
                None
            }
            Err(err) => Some(format!(
                "No se pudieron leer los ajustes ({err}); se usan los de fábrica."
            )),
        };
        self.settings_store = store;
        self.secrets = secrets;
        (self, problem)
    }

    /// Vuelve a leer qué claves hay en el llavero (enmascaradas).
    pub fn refresh_keys(&mut self) {
        let mut keys = settings_screen::KeyStatus::default();
        for provider in [ProviderKind::Anthropic, ProviderKind::OpenAi] {
            let Some(name) = provider.secret_key() else {
                continue;
            };
            let masked = match self.secrets.get(name) {
                Ok(secret) => secret.map(|s| s.masked()),
                Err(err) => {
                    keys.error = Some(format!("No se pudo leer el llavero: {err}"));
                    None
                }
            };
            match provider {
                ProviderKind::Anthropic => keys.anthropic = masked,
                _ => keys.openai = masked,
            }
        }
        self.keys = keys;
    }

    /// Guarda `settings` (ya validados) si cambian algo.
    fn update_settings(&mut self, settings: Settings) -> Outcome {
        let settings = settings.validated();
        if settings == self.settings {
            return Outcome::default();
        }
        match self.settings_store.save(&settings) {
            Ok(()) => {
                self.settings = settings;
                Outcome::default()
            }
            Err(err) => Outcome::error(format!("No se pudieron guardar los ajustes: {err}")),
        }
    }

    /// Crea la animación del dibujo: su receta (o sus píxeles) como una
    /// pieza, un reposo que la reproduce y una acción para empezar.
    fn create_animation(&mut self) -> Outcome {
        let document = match self.runtime.block_on(self.session.snapshot()) {
            Ok(document) => document,
            Err(err) => return Outcome::error(err.to_string()),
        };
        if document.animation.is_some() {
            return Outcome::default();
        }
        let recipe_lines: Vec<String> = document
            .recipe()
            .to_string()
            .lines()
            .map(String::from)
            .collect();
        let project = new_project(ProjectOptions {
            name: document.name.clone(),
            base: Image::from_canvas(document.canvas()),
            recipe_lines,
            cell: (0, 0),
            anchor: "bottom".into(),
            actions: Vec::new(),
        });
        let source = match project {
            Ok((source, _)) => source,
            Err(err) => return Outcome::error(format!("No se pudo crear la animación: {err}")),
        };
        let edit = AnimEdit::Set(Some(Animation::new(source)));
        match self
            .runtime
            .block_on(self.session.edit_animation(edit, Origin::User))
        {
            Ok(_) => {
                Outcome::notice("Animación creada: pídele al agente las acciones o edita anim.js")
            }
            Err(err) => Outcome::error(err.to_string()),
        }
    }

    /// Crea el dibujo vectorial del documento: un `dibujo.js` de 1024 × 1024
    /// en blanco (un paso de deshacer).
    fn create_vector(&mut self) -> Outcome {
        self.create_vector_sized(1024, 1024, false)
    }

    /// Crea el dibujo vectorial en blanco de `width × height` (si no lo hay).
    fn create_vector_sized(&mut self, width: u32, height: u32, transparent: bool) -> Outcome {
        if self.session.view().borrow().vector.is_some() {
            return Outcome::default();
        }
        let source = lume_domain::vec::template(width, height, transparent);
        let edit = VecEdit::Set(Some(VectorDrawing::new(source)));
        match self
            .runtime
            .block_on(self.session.edit_vector(edit, Origin::User))
        {
            Ok(_) => Outcome::notice("Dibujo vectorial creado: pídele al agente lo que quieras"),
            Err(err) => Outcome::error(err.to_string()),
        }
    }

    /// ¿Hay cambios sin guardar?
    pub fn is_dirty(&self) -> bool {
        self.saved_revision != Some(self.session.view().borrow().revision)
    }

    /// Nombre del archivo para la cabecera («pocion_morada.lume»).
    pub fn file_name(&self) -> String {
        match &self.path {
            Some(path) => path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            None => format!("{}.lume", self.session.view().borrow().name),
        }
    }

    /// Atiende un intent. Los cambios del dibujo se envían sin esperar; los de
    /// archivo esperan a la sesión (son rápidos). No se puede llamar desde
    /// dentro del runtime de tokio.
    pub fn handle(&mut self, intent: Intent) -> Outcome {
        let session = self.session.clone();
        match intent {
            Intent::Apply { command, label } => {
                self.runtime.spawn(async move {
                    let _ = session.apply_labeled(command, Origin::User, label).await;
                });
            }
            Intent::Undo | Intent::Redo => {
                let line = if intent == Intent::Undo {
                    "undo"
                } else {
                    "redo"
                };
                self.runtime.spawn(async move {
                    let _ = session.line(line, Origin::User, None).await;
                });
            }
            Intent::UndoSteps(steps) => {
                self.runtime.spawn(async move {
                    for _ in 0..steps {
                        let _ = session.line("undo", Origin::User, None).await;
                    }
                });
            }
            Intent::NewDocument {
                width,
                height,
                palette,
            } => {
                let mut document =
                    match Document::new("sin título", i64::from(width), i64::from(height)) {
                        Ok(document) => document,
                        Err(err) => return Outcome::error(err.message),
                    };
                document.palette = palette.as_deref().and_then(lume_domain::Palette::fixed);
                return self.replace(document, None, true);
            }
            Intent::OpenRecent(path) | Intent::OpenPath(path) => return self.open(path),
            Intent::OpenFile => {
                if let Some(path) = self.dialogs.open_lume() {
                    return self.open(path);
                }
            }
            Intent::Save => match self.path.clone() {
                Some(path) => return self.save_to(path),
                None => return self.handle(Intent::SaveAs),
            },
            Intent::SaveAs => {
                let name = self.session.view().borrow().name.clone();
                if let Some(path) = self.dialogs.save_lume(&name) {
                    return self.save_to(path);
                }
            }
            Intent::SaveTo(path) => return self.save_to(path),
            Intent::ExportPng { scale } => {
                let name = self.session.view().borrow().name.clone();
                if let Some(path) = self.dialogs.save_png(&name) {
                    return self.export(path, scale);
                }
            }
            Intent::ExportPngTo { path, scale } => return self.export(path, scale),
            Intent::ImportPxcli => {
                if let Some(dir) = self.dialogs.pick_folder() {
                    return self.import(dir);
                }
            }
            Intent::ImportPxcliFrom(dir) => return self.import(dir),
            Intent::GoHome => {
                return Outcome {
                    screen: Some(Screen::Home),
                    ..Outcome::default()
                };
            }
            Intent::OpenSettings => {
                self.refresh_keys();
                return Outcome {
                    screen: Some(Screen::Settings),
                    ..Outcome::default()
                };
            }
            Intent::UpdateSettings(settings) => return self.update_settings(settings),
            Intent::ShowCommands(on) => {
                let settings = Settings {
                    show_commands: on,
                    ..self.settings.clone()
                };
                return self.update_settings(settings);
            }
            Intent::DrawSpeed(draw_speed) => {
                let settings = Settings {
                    draw_speed,
                    ..self.settings.clone()
                };
                return self.update_settings(settings);
            }
            Intent::SaveApiKey { provider, key } => {
                let Some(name) = provider.secret_key() else {
                    return Outcome::default();
                };
                let key = lume_app::ports::Secret::new(key.expose().trim());
                if key.expose().is_empty() {
                    return Outcome::error("No se pudo guardar: la clave está vacía.");
                }
                let result = self.secrets.set(name, &key);
                self.refresh_keys();
                return match result {
                    Ok(()) => Outcome::notice("Clave guardada en el llavero del sistema."),
                    Err(err) => Outcome::error(format!("No se pudo guardar la clave: {err}")),
                };
            }
            Intent::DeleteApiKey(provider) => {
                let Some(name) = provider.secret_key() else {
                    return Outcome::default();
                };
                let result = self.secrets.delete(name);
                self.refresh_keys();
                if let Err(err) = result {
                    return Outcome::error(format!("No se pudo quitar la clave: {err}"));
                }
                // Sin clave no puede ser el proveedor por defecto.
                if self.settings.provider == provider {
                    let settings = Settings {
                        provider: ProviderKind::ClaudeCode,
                        ..self.settings.clone()
                    };
                    let outcome = self.update_settings(settings);
                    if outcome.error.is_some() {
                        return outcome;
                    }
                }
                return Outcome::notice("Clave quitada del llavero.");
            }
            // `Export` abre el diálogo de exportar y `CloseSettings` vuelve a la
            // pantalla de antes (los atiende la ventana); los otros modos llegan
            // en las fases 8 y 9 y el agente en la 5.
            Intent::Export
            | Intent::CloseSettings
            | Intent::SwitchMode(_)
            | Intent::Describe { .. }
            | Intent::AgentSend { .. }
            | Intent::AgentStop
            | Intent::AgentAttach
            | Intent::AgentPasteImage(_)
            | Intent::SelectTab(_)
            | Intent::ExportVector { .. }
            | Intent::ExportRigAction { .. }
            | Intent::RenameDocument(_)
            | Intent::CloseTab(_)
            | Intent::ShowVersion(_)
            | Intent::RestoreVersion(_)
            | Intent::ClosePreview
            | Intent::OpenJob(_)
            | Intent::OpenMain
            | Intent::AgentDraft(_)
            | Intent::RequestVariants { .. }
            | Intent::ChooseVariant(_)
            | Intent::MixVariants(_)
            | Intent::ShowVariants(_)
            | Intent::ExportAnimation { .. }
            | Intent::VectorZone
            | Intent::VectorSelect
            | Intent::RunCorrections
            | Intent::ReplayVector => {}
            Intent::CreateAnimation => return self.create_animation(),
            Intent::CreateVector => return self.create_vector(),
            Intent::NewVector {
                width,
                height,
                transparent,
            } => return self.create_vector_sized(width, height, transparent),
            Intent::AddVectorCode(code) => {
                let Some(drawing) = self.session.view().borrow().vector.clone() else {
                    return Outcome::error("Este documento no tiene dibujo vectorial");
                };
                let (source, _) = vec_draw::append(&drawing.source, &code);
                let edit = VecEdit::Source(source);
                if let Err(err) = self
                    .runtime
                    .block_on(session.edit_vector(edit, Origin::User))
                {
                    return Outcome::error(err.to_string());
                }
            }
            Intent::SetFrameDuration { action, frame, ms } => {
                let edit = AnimEdit::FrameDuration { action, frame, ms };
                if let Err(err) = self
                    .runtime
                    .block_on(session.edit_animation(edit, Origin::User))
                {
                    return Outcome::error(err.to_string());
                }
            }
        }
        Outcome::default()
    }

    /// Pone `document` en la sesión. `clean`: sin cambios por guardar.
    fn replace(
        &mut self,
        document: Document,
        path: Option<std::path::PathBuf>,
        clean: bool,
    ) -> Outcome {
        match self
            .runtime
            .block_on(self.session.replace_document(document))
        {
            Ok(()) => {
                self.path = path;
                self.saved_revision = clean.then(|| self.session.view().borrow().revision);
                Outcome::studio()
            }
            Err(err) => Outcome::error(err.to_string()),
        }
    }

    /// Pone `document` en la sesión (al cambiar de pestaña).
    fn swap_in(
        &mut self,
        document: Document,
        path: Option<std::path::PathBuf>,
        clean: bool,
    ) -> Outcome {
        self.replace(document, path, clean)
    }

    fn open(&mut self, path: std::path::PathBuf) -> Outcome {
        match self.store.open_document(&path) {
            Ok(document) => {
                if let Some(recents) = &self.recents {
                    let _ = recents.touch(&path, now_secs());
                }
                self.replace(document, Some(path), true)
            }
            Err(err) => Outcome::error(format!(
                "No se pudo abrir {}: {}",
                path.display(),
                err.message
            )),
        }
    }

    pub(crate) fn save_to(&mut self, path: std::path::PathBuf) -> Outcome {
        let name = path
            .file_stem()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "sin título".into());
        let result = self.runtime.block_on(async {
            self.session.set_name(name).await?;
            let revision = self.session.view().borrow().revision;
            let document = self.session.snapshot().await?;
            Ok::<_, lume_app::AppError>((document, revision))
        });
        let (document, revision) = match result {
            Ok(saved) => saved,
            Err(err) => return Outcome::error(err.to_string()),
        };
        if let Err(err) = self.store.save_document(&document, &path) {
            return Outcome::error(format!(
                "No se pudo guardar {}: {}",
                path.display(),
                err.message
            ));
        }
        if let Some(recents) = &self.recents {
            let _ = recents.touch(&path, now_secs());
        }
        self.saved_revision = Some(revision);
        let notice = format!("Guardado en {}", path.display());
        self.path = Some(path);
        Outcome::notice(notice)
    }

    fn export(&self, path: std::path::PathBuf, scale: u32) -> Outcome {
        let canvas = self.session.view().borrow().canvas.clone();
        match self.store.export_png(&canvas, &path, scale) {
            Ok(()) => Outcome::notice(format!("Exportado a {} ({}×)", path.display(), scale)),
            Err(err) => Outcome::error(format!(
                "No se pudo exportar {}: {}",
                path.display(),
                err.message
            )),
        }
    }

    fn import(&mut self, dir: std::path::PathBuf) -> Outcome {
        match self.store.import_pxcli_run(&dir) {
            // Importado: aún no tiene archivo y está por guardar.
            Ok(document) => self.replace(document, None, false),
            Err(err) => Outcome::error(format!(
                "No se pudo importar {}: {}",
                dir.display(),
                err.message
            )),
        }
    }
}

/// Qué contestó el usuario al aviso de cambios sin guardar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsavedChoice {
    Save,
    Discard,
    Cancel,
}

/// Aviso «¿Guardar los cambios?» (ventana modal). Devuelve la respuesta, si
/// la hay.
pub fn unsaved_dialog(ctx: &egui::Context, file_name: &str) -> Option<UnsavedChoice> {
    let mut choice = None;
    let modal = egui::Modal::new(egui::Id::unique("lumeart-sin-guardar")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.label(
            egui::RichText::new("¿Guardar los cambios?")
                .font(theme::semibold(16.0))
                .color(theme::TEXT),
        );
        ui.label(
            egui::RichText::new(format!(
                "{file_name} tiene cambios sin guardar. Si no los guardas, se perderán."
            ))
            .size(13.0)
            .color(theme::TEXT_2),
        );
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if ui.button("Cancelar").clicked() {
                choice = Some(UnsavedChoice::Cancel);
            }
            if ui.button("No guardar").clicked() {
                choice = Some(UnsavedChoice::Discard);
            }
            let save = egui::Button::new(egui::RichText::new("Guardar").color(theme::ACCENT_INK))
                .fill(theme::ACCENT);
            if ui.add(save).clicked() {
                choice = Some(UnsavedChoice::Save);
            }
        });
    });
    if modal.should_close() && choice.is_none() {
        choice = Some(UnsavedChoice::Cancel);
    }
    choice
}

/// Un documento abierto en la ventana que no es el que se ve: el dibujo
/// propio o un encargo.
struct OpenDoc {
    job: Option<ArtId>,
    session: SessionHandle,
    path: Option<std::path::PathBuf>,
    saved_revision: Option<u64>,
    panel: agent_panel::AgentPanel,
    /// Entradas del registro del encargo ya pasadas al panel.
    seen: usize,
    /// Revisión de la última vez que el encargo escribió sus archivos.
    written: Option<u64>,
}

impl OpenDoc {
    fn is_dirty(&self) -> bool {
        self.saved_revision != Some(self.session.view().borrow().revision)
    }
}

/// Una pestaña de un documento propio, con su id fijo (el `tab` del MCP) y
/// su sesión, siempre viva: cada pestaña es un documento independiente con
/// su propio agente, y se puede dibujar en varias a la vez. De la activa, lo
/// que vale es lo de `controller` y `shell` (aquí se guarda al dejarla).
struct DocTab {
    id: u64,
    session: SessionHandle,
    saved_revision: Option<u64>,
    path: Option<std::path::PathBuf>,
    mode: Mode,
    panel: agent_panel::AgentPanel,
    code_log: panel_tabs::CodeLog,
    thumbs: panel_tabs::HistoryThumbs,
    canvas: canvas_view::CanvasView,
    /// Su cola de correcciones del modo vector.
    corrections: Vec<shell::ZoneState>,
    /// Su agente (mientras no se ve).
    agent: Option<TabAgent>,
    /// Enseña Inicio (aún no tiene documento propio).
    home: bool,
}

/// El panel del agente de una pestaña nueva.
fn fresh_panel() -> agent_panel::AgentPanel {
    let mut panel = agent_panel::AgentPanel::default();
    panel.continue_drawing = true;
    panel
}

impl DocTab {
    fn new(id: u64, session: SessionHandle, saved_revision: Option<u64>) -> Self {
        Self {
            id,
            session,
            saved_revision,
            path: None,
            mode: Mode::Pixel,
            panel: fresh_panel(),
            code_log: Default::default(),
            thumbs: Default::default(),
            canvas: Default::default(),
            corrections: Vec::new(),
            agent: None,
            home: false,
        }
    }

    /// ¿Tiene cambios sin guardar?
    fn is_dirty(&self) -> bool {
        self.saved_revision != Some(self.session.view().borrow().revision)
    }

    /// ¿Su agente está dibujando?
    fn busy(&self) -> bool {
        self.agent.as_ref().is_some_and(|a| a.sup.running())
    }

    /// Su nombre en la tira («pocion.lume»).
    fn title(&self) -> String {
        if self.home {
            return "Inicio".into();
        }
        match &self.path {
            Some(p) => p
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default(),
            None => format!("{}.lume", self.session.view().borrow().name),
        }
    }
}

/// El agente de una pestaña y lo que lo mantiene vivo (su API propia).
struct TabAgent {
    sup: agent_run::AgentSupervisor,
    guard: Option<Box<dyn std::any::Any + Send + Sync>>,
}

/// La aplicación de la ventana: la pantalla de inicio o el studio.
pub struct StudioApp {
    /// Pestañas de documentos propios y cuál se ve.
    tabs: Vec<DocTab>,
    active_tab: usize,
    /// Pestaña que se quiere cerrar y tiene cambios (se pregunta).
    closing_tab: Option<usize>,
    /// El aviso de la barra de estado y desde cuándo se ve (se va solo).
    notice_since: Option<(String, f64)>,
    /// Peticiones del MCP a la ventana.
    window_inbox: Option<tokio::sync::mpsc::UnboundedReceiver<lume_app::studio::WindowCall>>,
    /// Autoguardado: carpeta, cuándo se hizo y qué había entonces.
    autosave: Option<std::path::PathBuf>,
    last_autosave: f64,
    autosaved: Option<u64>,
    pub controller: AppController,
    /// El agente de la pestaña activa.
    pub agent: agent_run::AgentSupervisor,
    /// Lo que mantiene vivo el agente de la pestaña activa si dibuja en una
    /// sesión propia (su API y su archivo de sesión).
    agent_guard: Option<Box<dyn std::any::Any + Send + Sync>>,
    /// El agente de la sesión de la ventana.
    window_backend: Option<Arc<dyn AgentBackend>>,
    /// Monta agentes para las pestañas con sesión propia.
    tab_agents: Option<Arc<dyn lume_app::ArtAgents>>,
    tab_runs: u64,
    /// Id de la próxima pestaña.
    next_tab_id: u64,
    /// Dónde se publican las sesiones de las pestañas (para la API).
    tab_sessions: Option<lume_app::studio::TabSessions>,
    /// Encargos (`create_art`) que se ven como pestañas, con su id de
    /// pestaña, y los que el usuario cerró.
    job_tabs: Vec<(ArtId, u64)>,
    hidden_jobs: std::collections::HashSet<ArtId>,
    /// Modo del dibujo propio mientras se ve un encargo.
    mode_before_job: Option<Mode>,
    /// Encargos que llegan por MCP (si esta ventana los acepta).
    pub art: Option<Arc<LocalArt>>,
    /// El encargo que se ve (`None`: el dibujo propio).
    pub viewing: Option<ArtId>,
    main_session: SessionHandle,
    /// Los documentos abiertos que no se ven.
    stash: Vec<OpenDoc>,
    seen: usize,
    written: Option<u64>,
    /// Encargos cuya sesión ya repinta la ventana al cambiar.
    watched: std::collections::HashSet<ArtId>,
    /// Motor de animación y el cálculo en marcha (clave y resultado).
    anim_engine: Option<Arc<dyn AnimEngine>>,
    anim_job: Option<(
        u64,
        std::sync::mpsc::Receiver<lume_domain::anim::AnimResult>,
    )>,
    /// Clave de lo que está dibujado y revisión del documento en que se miró.
    anim_key: Option<u64>,
    anim_seen: Option<(u64, usize, u64)>,
    /// Variantes de la animación y lo que ya se dibujó de cada una.
    pub variants: Option<Arc<lume_app::Variants>>,
    variant_jobs: Vec<(
        char,
        u64,
        std::sync::mpsc::Receiver<lume_domain::anim::AnimResult>,
    )>,
    variant_rendered: std::collections::HashMap<char, u64>,
    variant_ids: u64,
    /// Motor, rasterizador y buscador de formas del modo vector.
    vec_services: Option<VecServices>,
    /// El cálculo del dibujo en marcha (clave y resultado) y la clave de lo
    /// que se ve.
    vec_job: Option<(u64, std::sync::mpsc::Receiver<shell::VecView>)>,
    /// Piezas del esqueleto que se están pintando (animación vectorial).
    rig_job: Option<(u64, std::sync::mpsc::Receiver<shell::RigImages>)>,
    /// Trozos del dibujo (capa a capa) que se están pintando.
    slice_job: Option<(u64, std::sync::mpsc::Receiver<Vec<shell::SliceImage>>)>,
    vec_key: Option<u64>,
    /// `dibujo.js` y largo del historial la última vez que se miró (para
    /// saber qué líneas acaba de cambiar el agente).
    vec_seen: Option<(Option<Arc<VectorDrawing>>, usize)>,
    /// «Repetir trazado» en marcha: cómo pararlo y sus pasos.
    vec_replay: Option<(
        Arc<std::sync::atomic::AtomicBool>,
        std::sync::mpsc::Receiver<shell::ReplayFrame>,
    )>,
    vec_ids: u64,
    ctx: egui::Context,
    stop: CancellationToken,
    pub screen: Screen,
    pub home: HomeScreen,
    pub shell: AppShell,
    pub settings_screen: SettingsScreen,
    /// Ajustes abiertos (en su propia ventana).
    settings_open: bool,
    recent_list: Vec<RecentEntry>,
    /// Se está preguntando si guardar antes de cerrar.
    asking_to_close: bool,
    /// Ya se puede cerrar (guardado o descartado).
    close_allowed: bool,
    /// Ya se atendió `lumeart stop` (se pregunta una sola vez; si el usuario
    /// cancela, la ventana sigue abierta con su dibujo).
    stop_handled: bool,
}

impl StudioApp {
    /// `stop` cierra la ventana cuando se cancela (`lumeart stop`).
    pub fn new(cc: &eframe::CreationContext<'_>, options: StudioOptions) -> Self {
        theme::apply(&cc.egui_ctx);
        let StudioOptions {
            session,
            stop,
            runtime,
            store,
            recents,
            dialogs,
            settings,
            secrets,
            claude_path,
            agent,
            server,
            screen,
            art,
            anim_engine,
            variant_agents,
            vector,
            autosave,
            window,
        } = options;
        let tab_agents = variant_agents.clone();
        let window_backend = agent.clone();
        let variants = variant_agents.map(|agents| {
            let variants = Arc::new(lume_app::Variants::new(
                store.clone(),
                settings.clone(),
                agents,
            ));
            let ctx = cc.egui_ctx.clone();
            variants.set_notify(Arc::new(move || ctx.request_repaint()));
            variants
        });
        repaint_on_changes(&runtime, &session, cc.egui_ctx.clone(), stop.clone());
        if let Some(art) = &art {
            let ctx = cc.egui_ctx.clone();
            art.set_notify(Arc::new(move || ctx.request_repaint()));
        }
        let main_session = session.clone();
        let agent = agent_run::AgentSupervisor::new(
            agent,
            store.clone(),
            runtime.clone(),
            session.clone(),
            cc.egui_ctx.clone(),
        );
        let mut shell = AppShell::default();
        shell.server = server.clone();
        shell.agent.continue_drawing = true;
        let recent_list = recents.as_ref().map(|r| r.list()).unwrap_or_default();
        let (controller, problem) = AppController::new(session, runtime, store, recents, dialogs)
            .with_settings(settings, secrets);
        shell.notice = problem.clone();
        let mut settings_screen = SettingsScreen::new(claude_path, server);
        settings_screen.notice = problem;
        let mut app = Self {
            tabs: vec![DocTab::new(0, main_session.clone(), None)],
            active_tab: 0,
            closing_tab: None,
            notice_since: None,
            window_inbox: None,
            autosave: None,
            last_autosave: 0.0,
            autosaved: None,
            controller,
            agent,
            agent_guard: None,
            window_backend,
            tab_agents,
            tab_runs: 0,
            next_tab_id: 1,
            tab_sessions: None,
            job_tabs: Vec::new(),
            hidden_jobs: Default::default(),
            mode_before_job: None,
            art,
            viewing: None,
            main_session,
            stash: Vec::new(),
            seen: 0,
            written: None,
            watched: Default::default(),
            anim_engine,
            anim_job: None,
            anim_key: None,
            anim_seen: None,
            variants,
            variant_jobs: Vec::new(),
            variant_rendered: Default::default(),
            variant_ids: 0,
            vec_services: vector,
            vec_job: None,
            slice_job: None,
            rig_job: None,
            vec_key: None,
            vec_seen: None,
            vec_replay: None,
            vec_ids: 0,
            ctx: cc.egui_ctx.clone(),
            stop,
            screen,
            home: HomeScreen::default(),
            shell,
            settings_screen,
            settings_open: false,
            recent_list,
            asking_to_close: false,
            close_allowed: false,
            stop_handled: false,
        };
        app.autosave = autosave;
        // En la ventana, cada pestaña monta su agente al pedirle algo.
        app.agent = app.idle_agent(app.main_session.clone());
        if let Some(window) = window {
            app.tab_sessions = Some(window.tabs.clone());
            app.window_inbox = window.take_inbox();
            let ctx = cc.egui_ctx.clone();
            window.set_wake(Arc::new(move || ctx.request_repaint()));
        }
        if app.restore_autosave() {
            app.screen = Screen::Studio;
        } else if app.screen == Screen::Home {
            // Sin pestañas guardadas: Inicio en la primera pestaña.
            app.tabs[0].home = true;
            app.screen = Screen::Studio;
        }
        app
    }

    /// ¿Está abierta la ventana de Ajustes?
    pub fn settings_open(&self) -> bool {
        self.settings_open
    }

    /// ¿Está preguntando si guardar antes de cerrar?
    pub fn is_asking_to_close(&self) -> bool {
        self.asking_to_close
    }

    /// ¿Ya puede cerrarse (sin cambios, guardado o descartado)?
    pub fn close_allowed(&self) -> bool {
        self.close_allowed
    }

    /// Empieza una ejecución del agente con lo que pidió el usuario.
    fn start_agent(
        &mut self,
        prompt: &str,
        attachments: Vec<lume_app::ports::Attachment>,
        continuing: bool,
    ) {
        self.start_agent_as(prompt, prompt, attachments, continuing);
    }

    /// Empieza una ejecución del agente: `shown` es lo que se ve en la
    /// conversación y `goal` el encargo entero. En modo animación cambia
    /// `anim.js`; en modo vector, `dibujo.js` (que se crea si no lo hay).
    fn start_agent_as(
        &mut self,
        shown: &str,
        goal: &str,
        attachments: Vec<lume_app::ports::Attachment>,
        continuing: bool,
    ) {
        self.shell.agent.started(shown, &attachments);
        // El agente recibe también las referencias de mensajes anteriores.
        let attachments = self.shell.agent.remember_references(&attachments);
        let settings = self.controller.settings.clone();
        // Animar un dibujo vectorial es cambiar su esqueleto en dibujo.js.
        let vector_anim = self.shell.mode == Mode::Animation
            && self.controller.session.view().borrow().vector.is_some();
        let work = match self.shell.mode {
            Mode::Pixel => agent_run::AgentWork::Drawing,
            Mode::Animation if vector_anim => agent_run::AgentWork::Vector,
            Mode::Animation => agent_run::AgentWork::Animation,
            Mode::Vector => agent_run::AgentWork::Vector,
        };
        let goal = if vector_anim {
            format!("{VECTOR_ANIM_GOAL}{goal}")
        } else {
            goal.to_string()
        };
        let goal = goal.as_str();
        if work == agent_run::AgentWork::Vector {
            let outcome = self.controller.create_vector();
            if let Some(message) = outcome.error {
                self.shell.agent.finished(message, true);
                return;
            }
        }
        if !self.agent.has_backend() && !self.mount_tab_agent() {
            return;
        }
        if let Err(message) = self
            .agent
            .start(&settings, goal, attachments, continuing, work)
        {
            self.shell.agent.finished(message, true);
        }
    }

    /// Una imagen de referencia para el próximo envío (diálogo).
    fn attach_reference(&mut self) {
        let Some(path) = self.controller.dialogs.pick_image() else {
            return;
        };
        let media_type = match path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("jpg" | "jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "image/png",
        };
        match std::fs::read(&path) {
            Ok(data) => {
                let label = path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "referencia".into());
                self.add_reference(lume_app::ports::Attachment {
                    label,
                    media_type: media_type.into(),
                    data,
                });
            }
            Err(e) => {
                self.shell.notice = Some(format!("No se pudo leer {}: {e}", path.display()));
            }
        }
    }

    /// Añade una imagen de referencia (elegida o pegada) para el próximo
    /// envío, si no pasa de 8 MB.
    fn add_reference(&mut self, attachment: lume_app::ports::Attachment) {
        if attachment.data.len() > MAX_REFERENCE_BYTES {
            self.shell.notice = Some("La imagen pasa de 8 MB: usa una más pequeña.".into());
            return;
        }
        self.shell.agent.attachments.push(attachment);
    }

    /// Muestra una versión en el lienzo (sin restaurarla).
    fn show_version(&mut self, number: u32) {
        let document = self
            .controller
            .runtime
            .block_on(self.controller.session.snapshot());
        match document {
            Ok(document) => match document.version(number) {
                Ok(version) => {
                    self.shell.preview = Some(shell::VersionPreview {
                        number,
                        total: document.versions().len() as u32,
                        message: version.message.clone(),
                        canvas: Arc::new(version.canvas.clone()),
                        vector: version.vector.clone().map(Arc::new),
                    });
                }
                Err(err) => self.shell.notice = Some(err.message),
            },
            Err(err) => self.shell.notice = Some(err.to_string()),
        }
    }

    /// Enseña en la ventana el dibujo propio (`None`) o un encargo: guarda
    /// lo del que se deja (sesión, archivo, panel) y recupera lo del otro.
    fn switch_to(&mut self, target: Option<ArtId>) {
        self.screen = Screen::Studio;
        if target == self.viewing {
            return;
        }
        let session = match target {
            None => self.main_session.clone(),
            Some(id) => match self.art.as_ref().and_then(|art| art.session(id)) {
                Some(session) => session,
                None => {
                    self.shell.notice = Some(format!("No hay ningún encargo {id}."));
                    return;
                }
            },
        };
        // El modo del dibujo propio vuelve con él; un encargo se ve en el
        // suyo.
        if self.viewing.is_none() {
            self.mode_before_job = Some(self.shell.mode);
        }
        self.stash.push(OpenDoc {
            job: self.viewing,
            session: self.controller.session.clone(),
            path: self.controller.path.take(),
            saved_revision: self.controller.saved_revision,
            panel: std::mem::take(&mut self.shell.agent),
            seen: self.seen,
            written: self.written,
        });
        let doc = match self.stash.iter().position(|d| d.job == target) {
            Some(i) => self.stash.remove(i),
            None => {
                // Un encargo que se abre por primera vez.
                let status = target.and_then(|id| {
                    self.art
                        .as_ref()
                        .and_then(|art| art.jobs().into_iter().find(|s| s.id == id))
                });
                if let Some(id) = target
                    && self.watched.insert(id)
                {
                    repaint_on_changes(
                        &self.controller.runtime,
                        &session,
                        self.ctx.clone(),
                        self.stop.clone(),
                    );
                }
                let revision = session.view().borrow().revision;
                let written = status.as_ref().filter(|s| s.written).map(|s| s.revision);
                let mut panel = agent_panel::AgentPanel::default();
                panel.continue_drawing = true;
                OpenDoc {
                    job: target,
                    session,
                    path: status.map(|s| s.lume),
                    saved_revision: Some(written.unwrap_or(revision)),
                    panel,
                    seen: 0,
                    written,
                }
            }
        };
        self.controller.session = doc.session;
        self.controller.path = doc.path;
        self.controller.saved_revision = doc.saved_revision;
        self.shell.agent = doc.panel;
        self.seen = doc.seen;
        self.written = doc.written;
        self.viewing = target;
        self.shell.viewing_job = target;
        match target {
            Some(_) => {
                let vector = self.controller.session.view().borrow().vector.is_some();
                self.shell.mode = if vector { Mode::Vector } else { Mode::Pixel };
            }
            None => {
                if let Some(mode) = self.mode_before_job.take() {
                    self.shell.mode = mode;
                }
            }
        }
        self.shell.preview = None;
        self.shell.selection = None;
        self.shell.canvas.fit_pending = true;
        self.reset_vector();
    }

    /// La lista de encargos y, del que se ve, lo que pasó desde la última
    /// vez (al panel del agente).
    fn pump_jobs(&mut self) {
        let Some(art) = self.art.clone() else {
            return;
        };
        let statuses = art.jobs();
        let items: Vec<jobs::JobItem> = statuses
            .iter()
            .filter_map(|status| {
                let session = art.session(status.id)?;
                let vm = session.view().borrow().clone();
                Some(jobs::JobItem {
                    id: status.id,
                    name: status.name.clone(),
                    state: status.state,
                    state_text: jobs::state_text(status),
                    canvas: vm.canvas.clone(),
                    revision: vm.revision,
                })
            })
            .collect();
        self.home.jobs = items.clone();
        self.shell.jobs = items;

        // Cada encargo nuevo se ve como una pestaña (hasta que se cierra).
        for status in &statuses {
            if !self.hidden_jobs.contains(&status.id)
                && !self.job_tabs.iter().any(|(j, _)| *j == status.id)
            {
                self.job_tabs.push((status.id, self.next_tab_id));
                self.next_tab_id += 1;
            }
        }

        // Cuando un encargo escribe sus archivos, queda guardado.
        for status in statuses.iter().filter(|s| s.written) {
            if self.viewing == Some(status.id) {
                if self.written != Some(status.revision) {
                    self.written = Some(status.revision);
                    self.controller.saved_revision = Some(status.revision);
                }
            } else if let Some(doc) = self
                .stash
                .iter_mut()
                .find(|d| d.job == Some(status.id) && d.written != Some(status.revision))
            {
                doc.written = Some(status.revision);
                doc.saved_revision = Some(status.revision);
            }
        }

        let Some(id) = self.viewing else {
            return;
        };
        let entries = art.log(id, self.seen);
        self.seen += entries.len();
        let resume = self.controller.settings.context == lume_app::ContextPolicy::Resume;
        for entry in entries {
            match entry {
                JobEntry::Started { prompt } => self.shell.agent.started(&prompt, &[]),
                JobEntry::Loop(event) => {
                    // La miniatura de una vuelta es su versión.
                    let canvas = match &event {
                        lume_app::LoopEvent::TurnFinished { version, .. } => self
                            .controller
                            .runtime
                            .block_on(self.controller.session.snapshot())
                            .ok()
                            .and_then(|doc| {
                                doc.version(*version)
                                    .ok()
                                    .map(|v| Arc::new(canvas_image(&v.canvas)))
                            }),
                        _ => None,
                    };
                    self.shell.agent.apply_loop(&event, canvas, resume);
                }
                JobEntry::Finished { message, error } => {
                    self.shell.agent.finished(message, error);
                }
            }
        }
        let status = statuses.iter().find(|s| s.id == id);
        let running = status.is_some_and(|s| s.state == lume_app::ArtState::Running);
        self.shell.agent_view.running = running;
        self.shell.agent_status = status.filter(|_| running).map(|s| match s.turn {
            0 => "Dibujando…".to_string(),
            n => format!("Vuelta {n} de {}", s.max_turns),
        });
    }

    /// En modo animación, dibuja la animación del documento (en segundo
    /// plano) cuando cambian su código o su base.
    fn pump_anim(&mut self) {
        if self.shell.mode != Mode::Animation || self.screen != Screen::Studio {
            return;
        }
        if let Some((key, rx)) = &self.anim_job {
            match rx.try_recv() {
                Ok(result) => {
                    self.anim_key = Some(*key);
                    self.shell.anim.set_result(result);
                    self.anim_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.anim_job = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        let vm = self.controller.session.view().borrow().clone();
        let Some(animation) = vm.animation.clone() else {
            if self.shell.anim.result.is_some() {
                self.shell.anim.clear();
            }
            self.anim_key = None;
            return;
        };
        let Some(engine) = self.anim_engine.clone() else {
            self.shell.anim.no_engine = true;
            return;
        };
        let base = animation
            .base
            .clone()
            .unwrap_or_else(|| Image::from_canvas(&vm.canvas));
        // La clave (código y base) solo se recalcula cuando cambia el
        // `ViewModel` (otra revisión u otro documento).
        let seen = (vm.revision, Arc::as_ptr(&vm.canvas) as usize);
        let key = match self.anim_seen {
            Some((revision, canvas, key)) if (revision, canvas) == seen => key,
            _ => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                animation.source.hash(&mut hasher);
                base.hash(&mut hasher);
                let key = hasher.finish();
                self.anim_seen = Some((seen.0, seen.1, key));
                key
            }
        };
        if self.anim_key == Some(key) || self.anim_job.is_some() {
            return;
        }
        let (tx, rx) = std::sync::mpsc::channel();
        let source = animation.source.clone();
        let ctx = self.ctx.clone();
        self.controller.runtime.spawn_blocking(move || {
            let loader: lume_app::BaseLoader = Arc::new(move |_name: &str| Ok(base.clone()));
            let mut result = engine.render(&source, loader);
            lume_domain::anim::check::check(&mut result);
            let _ = tx.send(result);
            ctx.request_repaint();
        });
        self.anim_job = Some((key, rx));
    }

    /// En modo vector, ejecuta `dibujo.js` y lo pinta (en segundo plano)
    /// cuando cambian el código, las capas ocultas o la rejilla, y apunta
    /// qué líneas acaba de cambiar el agente.
    fn pump_vec(&mut self) {
        // También en Animación si el documento es vectorial (su esqueleto).
        let vector_doc = self.controller.session.view().borrow().vector.is_some();
        let wanted =
            self.shell.mode == Mode::Vector || (self.shell.mode == Mode::Animation && vector_doc);
        if !wanted || self.screen != Screen::Studio {
            return;
        }
        if let Some((_, rx)) = &self.rig_job {
            match rx.try_recv() {
                Ok(images) => {
                    self.shell.vanim.images = Some(Arc::new(images));
                    self.rig_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.rig_job = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        // En Animación: las piezas del esqueleto pintadas, una vez por versión.
        if self.shell.mode == Mode::Animation
            && self.rig_job.is_none()
            && let (Some(view), Some(services)) =
                (self.shell.vec.view.clone(), self.vec_services.clone())
            && view
                .result
                .rig
                .as_ref()
                .is_some_and(|r| !r.parts.is_empty())
            && self
                .shell
                .vanim
                .images
                .as_ref()
                .is_none_or(|i| i.key != view.key)
        {
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.ctx.clone();
            let key = view.key;
            self.controller.runtime.spawn_blocking(move || {
                let _ = tx.send(rig_images(&services, &view));
                ctx.request_repaint();
            });
            self.rig_job = Some((key, rx));
        }
        if let Some((key, rx)) = &self.vec_job {
            match rx.try_recv() {
                Ok(view) => {
                    self.vec_key = Some(*key);
                    self.shell.vec.set_view(view);
                    self.vec_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.vec_job = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        if let Some((key, rx)) = &self.slice_job {
            match rx.try_recv() {
                Ok(slices) => {
                    self.shell.vec.slices = Some((*key, Arc::new(slices)));
                    self.slice_job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Disconnected) => self.slice_job = None,
                Err(std::sync::mpsc::TryRecvError::Empty) => {}
            }
        }
        self.pump_replay();
        let vm = self.controller.session.view().borrow().clone();
        self.note_agent_lines(&vm);
        let drawing = match &self.shell.preview {
            Some(preview) => preview.vector.clone(),
            None => vm.vector.clone(),
        };
        let Some(drawing) = drawing else {
            if self.shell.vec.view.is_some() {
                self.shell.vec.clear();
            }
            self.vec_key = None;
            return;
        };
        let Some(services) = self.vec_services.clone() else {
            self.shell.vec.no_engine = true;
            return;
        };
        // La clave es solo el código: ocultar capas o ver la rejilla no
        // vuelve a pintar (las capas se componen con los trozos; la rejilla
        // la dibuja la ventana).
        let key = {
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            drawing.source.hash(&mut hasher);
            hasher.finish()
        };
        if self.vec_key == Some(key) && self.vec_job.is_none() {
            // El dibujo está: si aún no se pintó capa a capa, en segundo plano.
            let have = self
                .shell
                .vec
                .slices
                .as_ref()
                .is_some_and(|(k, _)| *k == key)
                || self.slice_job.as_ref().is_some_and(|(k, _)| *k == key);
            if let (false, Some(view)) = (have, self.shell.vec.view.clone())
                && !view.layers.is_empty()
            {
                let (tx, rx) = std::sync::mpsc::channel();
                let ctx = self.ctx.clone();
                self.controller.runtime.spawn_blocking(move || {
                    let _ = tx.send(slice_images(&services, &view));
                    ctx.request_repaint();
                });
                self.slice_job = Some((key, rx));
            }
            return;
        }
        if self.vec_job.is_some() {
            return;
        }
        // Lo que se repetía ya no es lo que hay.
        self.stop_replay();
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = self.ctx.clone();
        let source = drawing.source.clone();
        self.controller.runtime.spawn_blocking(move || {
            let _ = tx.send(vec_view(&services, source, key));
            ctx.request_repaint();
        });
        self.vec_job = Some((key, rx));
    }

    /// Si el último cambio de `dibujo.js` es del agente (y no se ha
    /// deshecho), sus líneas se resaltan en el panel de código.
    fn note_agent_lines(&mut self, vm: &lume_app::ViewModel) {
        let now = vm.vector.clone();
        let Some((before, _)) = self.vec_seen.replace((now.clone(), vm.history.len())) else {
            return;
        };
        let source = |d: &Option<Arc<VectorDrawing>>| d.as_ref().map(|d| d.source.clone());
        if source(&before) == source(&now) {
            return;
        }
        let by_agent = !vm.can_redo
            && matches!(
                vm.history.last().map(|h| h.origin),
                Some(Origin::Agent { .. })
            );
        self.shell.vec.agent_lines = match (by_agent, before, now) {
            (true, Some(b), Some(a)) => {
                lume_domain::vec::drawing::changed_lines(&b.source, &a.source)
            }
            (true, None, Some(a)) => Some((1, a.source.split('\n').count())),
            _ => None,
        };
        if let Some((first, _)) = self.shell.vec.agent_lines {
            self.shell.vec.scroll_to = Some(first);
        }
    }

    /// Los pasos de «Repetir trazado» que han llegado (se ve el último).
    fn pump_replay(&mut self) {
        let Some((_, rx)) = &self.vec_replay else {
            return;
        };
        loop {
            match rx.try_recv() {
                Ok(frame) => self.shell.vec.replay = Some(frame),
                Err(std::sync::mpsc::TryRecvError::Empty) => break,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.vec_replay = None;
                    self.shell.vec.replay = None;
                    break;
                }
            }
        }
    }

    fn stop_replay(&mut self) {
        if let Some((stop, _)) = self.vec_replay.take() {
            stop.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        self.shell.vec.replay = None;
    }

    /// Otro documento: se olvidan el dibujo, la zona y las capas ocultas.
    fn reset_vector(&mut self) {
        self.stop_replay();
        self.shell.vec.clear();
        self.shell.vec.hidden.clear();
        self.vec_key = None;
        self.vec_seen = None;
    }

    /// El lazo terminó: qué formas quedan debajo y sus líneas.
    fn vector_zone(&mut self) {
        let Some(points) = self.shell.vec.pending_zone.take() else {
            return;
        };
        let (Some(services), Some(view)) = (self.vec_services.clone(), self.shell.vec.view.clone())
        else {
            return;
        };
        let r = &view.result;
        match services
            .shapes
            .shapes_under(&r.svg, r.width as f64, r.height as f64, &points)
        {
            Ok(mut hits) => {
                // Lo que está en una capa oculta no se ve.
                let hidden: Vec<&str> = self
                    .shell
                    .vec
                    .hidden
                    .iter()
                    .filter_map(|&i| view.layers.get(i))
                    .map(|l| l.name.as_str())
                    .collect();
                hits.retain(|h| !hidden.contains(&h.layer.as_str()));
                self.shell.vec.selected = None;
                self.shell
                    .vec
                    .set_zone(lume_domain::vec::studio::Zone { points, hits });
            }
            Err(err) => self.shell.notice = Some(format!("No se pudo mirar la zona: {err}")),
        }
    }

    /// Clic con la herramienta de selección: la forma de encima y su línea.
    fn vector_select(&mut self) {
        let Some(point) = self.shell.vec.click.take() else {
            return;
        };
        let (Some(services), Some(view)) = (self.vec_services.clone(), self.shell.vec.view.clone())
        else {
            return;
        };
        let r = &view.result;
        match services
            .shapes
            .shape_at(&r.svg, r.width as f64, r.height as f64, point)
        {
            Ok(hit) => self.shell.vec.set_selected(hit),
            Err(err) => self.shell.notice = Some(format!("No se pudo mirar la forma: {err}")),
        }
    }

    /// «Ejecutar correcciones»: pide al agente todas las de la cola que
    /// esperan en una sola pasada. La conversación muestra cada una con su
    /// número y el encargo lleva además sus zonas, sus formas y sus líneas.
    fn run_corrections(&mut self) {
        if self.viewing.is_some() {
            self.shell.notice =
                Some("Las correcciones solo retocan el dibujo propio de la ventana".into());
            return;
        }
        if self.agent.running() {
            self.shell.notice = Some("Espera a que el agente termine".into());
            return;
        }
        let Some(view) = self.shell.vec.view.clone() else {
            return;
        };
        let pending: Vec<&mut shell::ZoneState> = self
            .shell
            .vec
            .corrections
            .iter_mut()
            .filter(|c| !c.sent)
            .collect();
        if pending.is_empty() {
            return;
        }
        let mut shown = if pending.len() == 1 {
            "Corrección:".to_string()
        } else {
            format!("{} correcciones:", pending.len())
        };
        let mut items: Vec<(String, lume_domain::vec::studio::Zone)> = Vec::new();
        for c in pending {
            c.sent = true;
            shown.push_str(&format!("\n{}. {}", c.number, c.text));
            items.push((c.text.clone(), c.zone.clone()));
        }
        let refs: Vec<(&str, &lume_domain::vec::studio::Zone)> =
            items.iter().map(|(t, z)| (t.as_str(), z)).collect();
        let goal =
            lume_domain::vec::studio::zones_prompt(&refs, view.result.width, view.result.height);
        self.start_agent_as(&shown, &goal, Vec::new(), true);
    }

    /// «Repetir trazado»: el dibujo se vuelve a pintar desde cero, forma a
    /// forma, en unos pocos pasos.
    fn replay_vector(&mut self) {
        let (Some(services), Some(view)) = (self.vec_services.clone(), self.shell.vec.view.clone())
        else {
            return;
        };
        self.stop_replay();
        let total = lume_domain::vec::studio::shape_count(&view.result.svg);
        if total == 0 {
            return;
        }
        let steps = total.min(REPLAY_STEPS);
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = stop.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let ctx = self.ctx.clone();
        self.vec_ids += 1;
        let base = self.vec_ids << 32;
        let svg = view.result.svg.clone();
        let (w, h) = (view.result.width as f64, view.result.height as f64);
        let options = raster_options(w, h);
        self.controller.runtime.spawn_blocking(move || {
            for step in 1..=steps {
                if flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let shown = (total * step).div_ceil(steps);
                let frame = lume_domain::vec::studio::shapes_until(&svg, shown);
                let Ok(image) = services.rasterizer.rasterize(&frame, w, h, &options) else {
                    break;
                };
                let frame = shell::ReplayFrame {
                    key: base + step as u64,
                    image: Arc::new(image),
                    shown,
                    total,
                };
                if tx.send(frame).is_err() {
                    break;
                }
                ctx.request_repaint();
                std::thread::sleep(std::time::Duration::from_millis(REPLAY_PAUSE_MS));
            }
            ctx.request_repaint();
        });
        self.vec_replay = Some((stop, rx));
    }

    /// Pide variantes de la animación (de la acción que se ve).
    fn request_variants(&mut self, prompt: &str) {
        let Some(variants) = self.variants.clone() else {
            self.shell.notice = Some("Esta ventana no puede pedir variantes".into());
            return;
        };
        let document = match self
            .controller
            .runtime
            .block_on(self.controller.session.snapshot())
        {
            Ok(document) => document,
            Err(err) => {
                self.shell.notice = Some(err.to_string());
                return;
            }
        };
        let action = self.shell.anim.action_name();
        match self.controller.runtime.block_on(variants.start(
            &document,
            prompt,
            3,
            action.as_deref(),
        )) {
            Ok(()) => {
                self.shell.agent.items.push(agent_panel::ConvItem::User {
                    text: prompt.to_string(),
                    attachments: vec!["3 variantes".into()],
                });
                self.variant_rendered.clear();
                self.variant_jobs.clear();
                self.shell.anim.variants.clear();
                self.shell.anim.showing_variants = true;
            }
            Err(err) => self.shell.notice = Some(format!("No se pudieron pedir variantes: {err}")),
        }
    }

    /// Elige una variante: su `anim.js` pasa a ser la animación del
    /// documento (un paso de deshacer) y queda una versión nueva.
    fn choose_variant(&mut self, index: usize) {
        let Some(card) = self.shell.anim.variants.get(index).cloned() else {
            return;
        };
        let Some(source) = card.source.clone() else {
            return;
        };
        let session = self.controller.session.clone();
        let chosen = self.controller.runtime.block_on(async {
            session
                .edit_animation(AnimEdit::Source(source), Origin::User)
                .await?;
            session
                .create_version(Origin::User, format!("Variante {} elegida", card.letter))
                .await
        });
        match chosen {
            Ok(number) => {
                self.shell.notice = Some(format!(
                    "Variante {} elegida: es la versión {number}",
                    card.letter
                ));
                if let Some(variants) = &self.variants {
                    variants.clear();
                }
                self.shell.anim.variants.clear();
                self.shell.anim.showing_variants = false;
                self.variant_rendered.clear();
                self.variant_jobs.clear();
            }
            Err(err) => self.shell.notice = Some(format!("No se pudo elegir: {err}")),
        }
    }

    /// Las variantes en pantalla: su estado y, las listas, dibujadas.
    fn pump_variants(&mut self) {
        let Some(variants) = self.variants.clone() else {
            return;
        };
        let list = variants.list();
        // Lo que acaba de dibujarse.
        let mut done = Vec::new();
        self.variant_jobs
            .retain(|(letter, key, rx)| match rx.try_recv() {
                Ok(result) => {
                    done.push((*letter, *key, result));
                    false
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => true,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => false,
            });
        let mut cards: Vec<shell::VariantCard> = Vec::new();
        for v in &list {
            let old = self
                .shell
                .anim
                .variants
                .iter()
                .find(|c| c.letter == v.letter)
                .cloned();
            let state = match &v.state {
                lume_app::VariantState::Running => shell::VariantCardState::Running,
                lume_app::VariantState::Ready(_) => shell::VariantCardState::Ready,
                lume_app::VariantState::Failed(e) => shell::VariantCardState::Failed(e.clone()),
            };
            let mut card = shell::VariantCard {
                letter: v.letter,
                mix: v.mix,
                state,
                summary: v.summary.clone(),
                source: v.source().map(str::to_string),
                result: old.as_ref().and_then(|c| c.result.clone()),
                id: old.as_ref().map_or(0, |c| c.id),
            };
            if let Some((_, _, result)) = done.iter().find(|(l, _, _)| *l == v.letter) {
                self.variant_ids += 1;
                card.id = self.variant_ids;
                card.result = Some(Arc::new(result.clone()));
            }
            cards.push(card);
        }
        for (letter, key, _) in &done {
            self.variant_rendered.insert(*letter, *key);
        }
        // Las listas que aún no se han dibujado.
        if let Some(engine) = self.anim_engine.clone() {
            let vm = self.controller.session.view().borrow().clone();
            for card in &cards {
                let Some(source) = card.source.clone() else {
                    continue;
                };
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                source.hash(&mut hasher);
                let key = hasher.finish();
                let busy = self
                    .variant_jobs
                    .iter()
                    .any(|(l, k, _)| *l == card.letter && *k == key);
                if self.variant_rendered.get(&card.letter) == Some(&key) || busy {
                    continue;
                }
                let base = vm
                    .animation
                    .as_ref()
                    .and_then(|a| a.base.clone())
                    .unwrap_or_else(|| Image::from_canvas(&vm.canvas));
                let (tx, rx) = std::sync::mpsc::channel();
                let engine = engine.clone();
                let ctx = self.ctx.clone();
                self.controller.runtime.spawn_blocking(move || {
                    let loader: lume_app::BaseLoader =
                        Arc::new(move |_name: &str| Ok(base.clone()));
                    let mut result = engine.render(&source, loader);
                    lume_domain::anim::check::check(&mut result);
                    let _ = tx.send(result);
                    ctx.request_repaint();
                });
                self.variant_jobs.push((card.letter, key, rx));
            }
        }
        self.shell.anim.variants = cards;
        // La pastilla de la cabecera.
        if !list.is_empty() && !self.agent.running() {
            let ready = list
                .iter()
                .filter(|v| matches!(v.state, lume_app::VariantState::Ready(_)))
                .count();
            self.shell.agent_status = Some(if ready == list.len() {
                format!("{ready} variantes listas")
            } else {
                format!("Variantes: {ready} de {} listas", list.len())
            });
        }
    }

    /// Exporta la animación a la carpeta que elija el usuario.
    fn export_animation(
        &mut self,
        format: lume_domain::anim::export::ExportFormat,
        scale: u32,
        file_name: &str,
        open_folder: bool,
    ) {
        use lume_domain::anim::export::{ExportFile, export_files};
        let Some(result) = self.shell.anim.result.clone() else {
            return;
        };
        let Some(folder) = self.controller.dialogs.pick_folder() else {
            return;
        };
        let vm = self.controller.session.view().borrow().clone();
        let durations: Vec<Vec<u32>> = result
            .actions
            .iter()
            .map(|a| {
                vm.animation
                    .as_ref()
                    .map(|anim| anim.durations_for(&a.spec))
                    .unwrap_or_default()
            })
            .collect();
        let name = if file_name.trim().is_empty() {
            lume_domain::anim::export::sheet_file_name(&result, scale)
        } else {
            file_name.trim().to_string()
        };
        let store = self.controller.store.clone();
        let files = export_files(&result, &durations, format, scale, &name);
        let count = files.len();
        for (name, file) in files {
            let bytes = match file {
                ExportFile::Png(img) => store.encode_image(&img),
                ExportFile::Gif {
                    frames,
                    durations,
                    looping,
                } => store.encode_gif(&frames, &durations, looping),
                ExportFile::Text(text) => Ok(text.into_bytes()),
            };
            if let Err(err) = bytes.and_then(|bytes| store.write_file(&folder.join(&name), &bytes))
            {
                self.shell.notice = Some(format!("No se pudo exportar {name}: {}", err.message));
                return;
            }
        }
        self.shell.notice = Some(format!(
            "Exportados {count} archivos en {}",
            folder.display()
        ));
        if open_folder {
            self.controller.dialogs.open_folder(&folder);
        }
    }

    /// ¿Hay algún documento abierto con cambios sin guardar?
    fn any_dirty(&self) -> bool {
        self.controller.is_dirty()
            || self.stash.iter().any(OpenDoc::is_dirty)
            || self
                .tabs
                .iter()
                .enumerate()
                .any(|(i, t)| i != self.active_tab && t.is_dirty())
    }

    /// Si el que se ve no tiene cambios, enseña uno que sí (para preguntar).
    fn focus_dirty(&mut self) {
        if self.controller.is_dirty() {
            return;
        }
        if let Some(job) = self.stash.iter().find(|d| d.is_dirty()).map(|d| d.job) {
            self.switch_to(job);
            return;
        }
        if let Some(i) =
            (0..self.tabs.len()).find(|&i| i != self.active_tab && self.tabs[i].is_dirty())
        {
            // Al cerrar la ventana el agente ya no importa.
            if self.park_active() {
                self.unpark(i);
            }
        }
    }

    // --- Pestañas -------------------------------------------------------

    /// ¿La pestaña activa tiene algo (archivo, cambios, historial o
    /// dibujo)? Si no, un documento nuevo se abre en ella.
    fn tab_in_use(&self) -> bool {
        let vm = self.controller.session.view().borrow().clone();
        self.controller.path.is_some()
            || self.controller.is_dirty()
            || !vm.history.is_empty()
            || vm.vector.is_some()
            || vm.animation.is_some()
            || vm.canvas.pixels().iter().any(|p| p.a > 0)
    }

    /// Un agente sin trabajo para `session`. En la ventana, cada pestaña
    /// monta el suyo al pedirle algo (con su propia API, para que no dibuje
    /// en otra); sin eso, solo la sesión de la ventana tiene el suyo.
    fn idle_agent(&self, session: SessionHandle) -> agent_run::AgentSupervisor {
        let backend = if self.tab_agents.is_none() && session.same(&self.main_session) {
            self.window_backend.clone()
        } else {
            None
        };
        agent_run::AgentSupervisor::new(
            backend,
            self.controller.store.clone(),
            self.controller.runtime.clone(),
            session,
            self.ctx.clone(),
        )
    }

    /// Monta el agente de la pestaña activa (con su API, para que no dibuje
    /// en otra pestaña). `false` si no se pudo.
    fn mount_tab_agent(&mut self) -> bool {
        let Some(agents) = self.tab_agents.clone() else {
            self.shell
                .agent
                .finished("Esta pestaña no tiene agente en esta ventana.", true);
            return false;
        };
        self.tab_runs += 1;
        let id = 1_000_000 + self.tab_runs;
        let session = self.controller.session.clone();
        match self
            .controller
            .runtime
            .block_on(agents.agent(session.clone(), id))
        {
            Ok(mounted) => {
                self.agent = agent_run::AgentSupervisor::new(
                    Some(mounted.backend),
                    self.controller.store.clone(),
                    self.controller.runtime.clone(),
                    session,
                    self.ctx.clone(),
                );
                self.agent_guard = Some(mounted.guard);
                true
            }
            Err(err) => {
                self.shell
                    .agent
                    .finished(format!("No se pudo preparar el agente: {err}"), true);
                false
            }
        }
    }

    /// Una sesión nueva con `document` (la ventana se repinta al cambiar).
    fn spawn_session(&self, document: Document) -> SessionHandle {
        let session = {
            let _guard = self.controller.runtime.enter();
            lume_app::Session::spawn(document, self.controller.store.clone()).handle
        };
        repaint_on_changes(
            &self.controller.runtime,
            &session,
            self.ctx.clone(),
            self.stop.clone(),
        );
        session
    }

    /// Añade una pestaña (al final, sin enseñarla) con `document` en su
    /// propia sesión. Devuelve su índice.
    fn push_tab(
        &mut self,
        document: Document,
        path: Option<std::path::PathBuf>,
        clean: bool,
    ) -> usize {
        let session = self.spawn_session(document);
        let revision = session.view().borrow().revision;
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        let mut tab = DocTab::new(id, session, clean.then_some(revision));
        tab.path = path;
        self.tabs.push(tab);
        self.tabs.len() - 1
    }

    /// Abre `document` en una pestaña nueva y la enseña. Devuelve su id.
    fn open_tab(
        &mut self,
        document: Document,
        path: Option<std::path::PathBuf>,
        clean: bool,
        mode: Mode,
    ) -> u64 {
        self.park_active();
        let i = self.push_tab(document, path, clean);
        self.tabs[i].mode = mode;
        self.unpark(i);
        self.tabs[i].id
    }

    /// Guarda la pestaña activa: lo que se ve de ella (modo, conversación,
    /// código, lienzo) y su agente, que sigue trabajando si lo estaba.
    fn park_active(&mut self) -> bool {
        let placeholder = self.idle_agent(self.controller.session.clone());
        let sup = std::mem::replace(&mut self.agent, placeholder);
        let guard = self.agent_guard.take();
        let tab = &mut self.tabs[self.active_tab];
        tab.session = self.controller.session.clone();
        tab.saved_revision = self.controller.saved_revision;
        tab.path = self.controller.path.clone();
        tab.agent = Some(TabAgent { sup, guard });
        tab.mode = self.shell.mode;
        tab.panel = std::mem::take(&mut self.shell.agent);
        tab.code_log = std::mem::take(&mut self.shell.code_log);
        tab.thumbs = std::mem::take(&mut self.shell.history_thumbs);
        tab.canvas = std::mem::take(&mut self.shell.canvas);
        tab.corrections = std::mem::take(&mut self.shell.vec.corrections);
        true
    }

    /// Enseña la pestaña `i` (sin guardar antes la activa: eso es
    /// `park_active`), con su sesión y su agente.
    fn unpark(&mut self, i: usize) {
        let tab = &mut self.tabs[i];
        self.controller.session = tab.session.clone();
        self.controller.path = tab.path.clone();
        self.controller.saved_revision = tab.saved_revision;
        self.shell.agent = std::mem::take(&mut tab.panel);
        self.shell.code_log = std::mem::take(&mut tab.code_log);
        self.shell.history_thumbs = std::mem::take(&mut tab.thumbs);
        self.shell.canvas = std::mem::take(&mut tab.canvas);
        self.shell.mode = tab.mode;
        let mut corrections = std::mem::take(&mut tab.corrections);
        let agent = tab.agent.take();
        let session = tab.session.clone();
        self.active_tab = i;
        match agent {
            Some(agent) => {
                self.agent = agent.sup;
                self.agent_guard = agent.guard;
            }
            None => {
                self.agent = self.idle_agent(session);
                self.agent_guard = None;
            }
        }
        self.shell.preview = None;
        self.shell.selection = None;
        self.shell.canvas.fit_pending = true;
        self.reset_vector();
        // Su cola de correcciones (lo que estaba en curso sigue en curso si
        // su agente sigue).
        if !self.agent.running() {
            for c in &mut corrections {
                c.sent = false;
            }
        }
        self.shell.vec.corrections = corrections;
    }

    /// Hace `f` con la pestaña `i` delante y vuelve a la que se veía (para
    /// las peticiones del MCP a otra pestaña).
    fn with_tab<T>(&mut self, i: usize, f: impl FnOnce(&mut Self) -> T) -> T {
        let original = self.active_tab;
        if i == original {
            return f(self);
        }
        self.park_active();
        self.unpark(i);
        let out = f(self);
        self.park_active();
        self.unpark(original.min(self.tabs.len() - 1));
        out
    }

    /// Publica las sesiones de las pestañas para la API (y la activa).
    fn publish_tabs(&self) {
        let Some(registry) = &self.tab_sessions else {
            return;
        };
        let mut list: Vec<(u64, SessionHandle)> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let session = if i == self.active_tab && self.viewing.is_none() {
                    self.controller.session.clone()
                } else {
                    t.session.clone()
                };
                (t.id, session)
            })
            .collect();
        if let Some(art) = &self.art {
            for &(job, id) in &self.job_tabs {
                if let Some(session) = art.session(job) {
                    list.push((id, session));
                }
            }
        }
        let active = match self.viewing {
            Some(job) => self
                .job_tabs
                .iter()
                .find(|(j, _)| *j == job)
                .map(|(_, id)| *id),
            None => self.tabs.get(self.active_tab).map(|t| t.id),
        };
        registry.publish(list, active);
    }

    // --- Peticiones del MCP -------------------------------------------------

    /// Atiende las peticiones que otros agentes mandan a la ventana.
    fn pump_window(&mut self) {
        let Some(inbox) = self.window_inbox.as_mut() else {
            return;
        };
        let mut calls = Vec::new();
        while let Ok(call) = inbox.try_recv() {
            calls.push(call);
        }
        for (request, tab, reply) in calls {
            let result = self
                .window_request(request, tab)
                .map(|target| self.window_state(target));
            let _ = reply.send(result);
        }
        self.publish_tabs();
    }

    /// A qué pestaña va una petición: (índice en `tabs`) o un encargo.
    fn find_tab(&self, id: u64) -> Option<Result<usize, ArtId>> {
        if let Some(i) = self.tabs.iter().position(|t| t.id == id) {
            return Some(Ok(i));
        }
        self.job_tabs
            .iter()
            .find(|(_, t)| *t == id)
            .map(|(job, _)| Err(*job))
    }

    /// Lo que se ve de la ventana para el MCP; `target` es la pestaña de la
    /// que habla (su agente, sus mensajes y su archivo).
    fn window_state(&self, target: Option<u64>) -> lume_app::studio::WindowState {
        use lume_app::studio::{DocKind, TabInfo, WindowState};
        let kind = |m: Mode| match m {
            Mode::Pixel => DocKind::Pixel,
            Mode::Vector => DocKind::Vector,
            Mode::Animation => DocKind::Animation,
        };
        let views = self.tab_views();
        let active_view = self.active_view();
        let tabs: Vec<TabInfo> = views
            .iter()
            .enumerate()
            .map(|(index, t)| TabInfo {
                id: t.id,
                index,
                title: t.title.clone(),
                mode: kind(t.mode),
                dirty: t.dirty,
                active: index == active_view,
                agent_running: t.busy,
                job: t.job,
            })
            .collect();
        let target = target.or_else(|| views.get(active_view).map(|t| t.id));
        let messages = |panel: &agent_panel::AgentPanel| -> Vec<String> {
            panel
                .items
                .iter()
                .rev()
                .filter_map(|i| match i {
                    agent_panel::ConvItem::Text(t) => Some(t.clone()),
                    agent_panel::ConvItem::End { text, .. } => Some(text.clone()),
                    _ => None,
                })
                .take(4)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        };
        let (agent_running, agent_messages, file) = match target.and_then(|t| self.find_tab(t)) {
            Some(Ok(i)) if i == self.active_tab && self.viewing.is_none() => (
                self.agent.running(),
                messages(&self.shell.agent),
                self.controller.path.clone(),
            ),
            Some(Ok(i)) => {
                let t = &self.tabs[i];
                let panel = if i == self.active_tab {
                    // Se ve un encargo: el panel propio está guardado aparte.
                    self.stash
                        .iter()
                        .find(|d| d.job.is_none())
                        .map(|d| &d.panel)
                        .unwrap_or(&t.panel)
                } else {
                    &t.panel
                };
                let running = if i == self.active_tab {
                    self.agent.running()
                } else {
                    t.busy()
                };
                (running, messages(panel), t.path.clone())
            }
            Some(Err(job)) => (
                tabs.iter().any(|t| t.job == Some(job) && t.agent_running),
                Vec::new(),
                None,
            ),
            None => (false, Vec::new(), None),
        };
        WindowState {
            tabs,
            tab: target,
            file: file.map(|p| p.display().to_string()),
            agent_running,
            agent_messages,
            notice: self.shell.notice.clone(),
        }
    }

    /// Abre `document` en una pestaña nueva (o en la actual si está vacía).
    /// Devuelve el id de la pestaña.
    fn open_in_tab(
        &mut self,
        document: Document,
        path: Option<std::path::PathBuf>,
        mode: Mode,
    ) -> u64 {
        if self.viewing.is_some() {
            self.switch_to(None);
        }
        let clean = path.is_some();
        self.screen = Screen::Studio;
        if self.tab_in_use() || self.agent.running() {
            return self.open_tab(document, path, clean, mode);
        }
        self.tabs[self.active_tab].home = false;
        let outcome = self.controller.swap_in(document, path, clean);
        if let Some(e) = outcome.error {
            self.shell.notice = Some(e);
        }
        self.shell.mode = mode;
        self.shell.preview = None;
        self.reset_vector();
        self.tabs[self.active_tab].id
    }

    /// Una petición del MCP a la pestaña `tab` (o a la activa). Devuelve la
    /// pestaña de la que habla la respuesta; los «no» vuelven como error con
    /// su motivo.
    fn window_request(
        &mut self,
        request: lume_app::studio::WindowRequest,
        tab: Option<u64>,
    ) -> Result<Option<u64>, String> {
        use lume_app::studio::{DocKind, WindowRequest};
        let mode_of = |k: DocKind| match k {
            DocKind::Pixel => Mode::Pixel,
            DocKind::Vector => Mode::Vector,
            DocKind::Animation => Mode::Animation,
        };
        // La pestaña pedida: índice de una propia o un encargo.
        let target =
            match tab {
                Some(id) => Some(self.find_tab(id).ok_or_else(|| {
                    format!("no hay pestaña {id}: mira las que hay con list_tabs")
                })?),
                None => None,
            };
        let job_only = "esa pestaña es un encargo de create_art: cámbialo con revise_art";
        // Índice de la pestaña propia a la que va (la activa si no se dijo).
        let doc_index = |target: &Option<Result<usize, ArtId>>| -> Result<usize, String> {
            match target {
                Some(Ok(i)) => Ok(*i),
                Some(Err(_)) => Err(job_only.to_string()),
                None => Ok(self.active_tab),
            }
        };
        match request {
            WindowRequest::State => Ok(tab),
            WindowRequest::New {
                doc,
                width,
                height,
                palette,
            } => {
                let document =
                    lume_app::studio::new_document(doc, width, height, palette.as_deref())
                        .map_err(|e| e.to_string())?;
                Ok(Some(self.open_in_tab(document, None, mode_of(doc))))
            }
            WindowRequest::Open { path } => {
                let document =
                    self.controller.store.open_document(&path).map_err(|e| {
                        format!("no se pudo abrir {}: {}", path.display(), e.message)
                    })?;
                if let Some(recents) = &self.controller.recents {
                    let _ = recents.touch(&path, now_secs());
                }
                let mode = if document.vector.is_some() {
                    Mode::Vector
                } else if document.animation.is_some() {
                    Mode::Animation
                } else {
                    Mode::Pixel
                };
                Ok(Some(self.open_in_tab(document, Some(path), mode)))
            }
            WindowRequest::Save { path } => {
                let i = doc_index(&target)?;
                let id = self.tabs[i].id;
                if i == self.active_tab && self.viewing.is_none() {
                    let path = path
                        .or_else(|| self.controller.path.clone())
                        .ok_or("el documento no tiene archivo: di dónde guardarlo (path)")?;
                    if let Some(e) = self.controller.save_to(path).error {
                        return Err(e);
                    }
                } else {
                    let t = &self.tabs[i];
                    let path = path
                        .or_else(|| t.path.clone())
                        .ok_or("el documento no tiene archivo: di dónde guardarlo (path)")?;
                    let document = self
                        .controller
                        .runtime
                        .block_on(t.session.snapshot())
                        .map_err(|e| e.to_string())?;
                    let revision = t.session.view().borrow().revision;
                    self.controller
                        .store
                        .save_document(&document, &path)
                        .map_err(|e| e.message)?;
                    if let Some(recents) = &self.controller.recents {
                        let _ = recents.touch(&path, now_secs());
                    }
                    let t = &mut self.tabs[i];
                    t.path = Some(path);
                    t.saved_revision = Some(revision);
                }
                Ok(Some(id))
            }
            WindowRequest::SelectTab { index } => {
                match target {
                    Some(Ok(i)) => self.select_tab(i),
                    Some(Err(job)) => self.switch_to(Some(job)),
                    None => {
                        let count = self.tabs.len() + self.job_tabs.len();
                        if index >= count {
                            return Err(format!("no hay pestaña en la posición {index}"));
                        }
                        self.select_view(index);
                    }
                }
                self.screen = Screen::Studio;
                Ok(self.tab_views().get(self.active_view()).map(|t| t.id))
            }
            WindowRequest::CloseTab { index } => {
                let view = match target {
                    Some(Ok(i)) => i,
                    Some(Err(job)) => {
                        let pos = self
                            .job_tabs
                            .iter()
                            .position(|(j, _)| *j == job)
                            .unwrap_or(0);
                        self.tabs.len() + pos
                    }
                    None => index,
                };
                if view >= self.tabs.len() + self.job_tabs.len() {
                    return Err(format!("no hay pestaña en la posición {view}"));
                }
                if view < self.tabs.len() {
                    let dirty = if view == self.active_tab {
                        self.controller.is_dirty()
                    } else {
                        self.tabs[view].is_dirty()
                    };
                    if dirty {
                        return Err(
                            "esa pestaña tiene cambios sin guardar: guárdala antes (save_document)"
                                .into(),
                        );
                    }
                }
                self.close_view(view, false);
                Ok(None)
            }
            WindowRequest::Rename { name } => {
                let i = doc_index(&target)?;
                let id = self.tabs[i].id;
                self.with_tab(i, |app| app.rename_document(&name));
                Ok(Some(id))
            }
            WindowRequest::SetMode { mode } => {
                let i = doc_index(&target)?;
                let id = self.tabs[i].id;
                let mode = mode_of(mode);
                if i == self.active_tab && self.viewing.is_none() {
                    self.shell.mode = mode;
                    if mode == Mode::Vector {
                        self.dispatch(vec![Intent::CreateVector]);
                    }
                    self.screen = Screen::Studio;
                } else {
                    self.with_tab(i, |app| {
                        app.shell.mode = mode;
                        if mode == Mode::Vector {
                            app.dispatch(vec![Intent::CreateVector]);
                        }
                    });
                }
                Ok(Some(id))
            }
            WindowRequest::AskAgent { prompt } => {
                let i = doc_index(&target)?;
                let id = self.tabs[i].id;
                let running = if i == self.active_tab {
                    self.agent.running()
                } else {
                    self.tabs[i].busy()
                };
                if running {
                    return Err(format!(
                        "el agente de la pestaña {id} está trabajando: espera a que termine (window_agent_status) o páralo (stop_window_agent)"
                    ));
                }
                if self.viewing.is_some() && i == self.active_tab {
                    self.switch_to(None);
                }
                self.with_tab(i, |app| {
                    app.dispatch(vec![Intent::AgentSend {
                        prompt,
                        attachments: Vec::new(),
                        continuing: true,
                    }]);
                });
                Ok(Some(id))
            }
            WindowRequest::StopAgent => {
                let i = doc_index(&target)?;
                let id = self.tabs[i].id;
                if i == self.active_tab {
                    self.dispatch(vec![Intent::AgentStop]);
                } else if let Some(agent) = &self.tabs[i].agent {
                    agent.sup.stop();
                }
                Ok(Some(id))
            }
        }
    }

    // --- Autoguardado ---------------------------------------------------

    /// Guarda todas las pestañas en la carpeta de autoguardado (si algo
    /// cambió desde la última vez) con un índice para restaurarlas.
    fn autosave_now(&mut self) {
        let Some(dir) = self.autosave.clone() else {
            return;
        };
        // Viendo un encargo, la sesión no es la de la pestaña activa.
        if self.viewing.is_some() {
            return;
        }
        let signature = {
            use std::hash::{Hash, Hasher};
            let mut h = std::collections::hash_map::DefaultHasher::new();
            self.controller
                .session
                .view()
                .borrow()
                .revision
                .hash(&mut h);
            self.controller.file_name().hash(&mut h);
            self.active_tab.hash(&mut h);
            for t in &self.tabs {
                (t.is_dirty(), &t.path, t.mode as u8).hash(&mut h);
                t.session.view().borrow().revision.hash(&mut h);
            }
            h.finish()
        };
        if self.autosaved == Some(signature) {
            return;
        }
        if std::fs::create_dir_all(&dir).is_err() {
            return;
        }
        let mut entries = Vec::new();
        let mut keep = Vec::new();
        for i in 0..self.tabs.len() {
            let (document, path, dirty) = if i == self.active_tab {
                match self
                    .controller
                    .runtime
                    .block_on(self.controller.session.snapshot())
                {
                    Ok(d) => (d, self.controller.path.clone(), self.controller.is_dirty()),
                    Err(_) => continue,
                }
            } else {
                let tab = &self.tabs[i];
                match self.controller.runtime.block_on(tab.session.snapshot()) {
                    Ok(d) => (d, tab.path.clone(), tab.is_dirty()),
                    Err(_) => continue,
                }
            };
            // Un lienzo en blanco sin nada no vale la pena.
            let empty = path.is_none()
                && document.history().entries().is_empty()
                && document.vector.is_none()
                && document.canvas().pixels().iter().all(|p| p.a == 0);
            if empty {
                continue;
            }
            let file = format!("pestana-{}.lume", entries.len());
            if self
                .controller
                .store
                .save_document(&document, &dir.join(&file))
                .is_err()
            {
                continue;
            }
            let mode = if i == self.active_tab {
                self.shell.mode
            } else {
                self.tabs[i].mode
            };
            entries.push(serde_json::json!({
                "file": file,
                "path": path.as_ref().map(|p| p.display().to_string()),
                "dirty": dirty,
                "mode": match mode { Mode::Pixel => "pixel", Mode::Vector => "vector", Mode::Animation => "animacion" },
                "active": i == self.active_tab,
            }));
            keep.push(file);
        }
        let index = serde_json::json!({ "tabs": entries });
        let tmp = dir.join("pestanas.json.tmp");
        if std::fs::write(&tmp, index.to_string()).is_ok() {
            let _ = std::fs::rename(&tmp, dir.join("pestanas.json"));
        }
        // Fuera las copias de pestañas que ya no están.
        if let Ok(read) = std::fs::read_dir(&dir) {
            for entry in read.flatten() {
                let name = entry.file_name().to_string_lossy().into_owned();
                if name.starts_with("pestana-") && !keep.contains(&name) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
        self.autosaved = Some(signature);
    }

    /// Abre las pestañas autoguardadas. Devuelve si había alguna.
    fn restore_autosave(&mut self) -> bool {
        let Some(dir) = self.autosave.clone() else {
            return false;
        };
        let Ok(text) = std::fs::read_to_string(dir.join("pestanas.json")) else {
            return false;
        };
        let Ok(index) = serde_json::from_str::<serde_json::Value>(&text) else {
            return false;
        };
        // (documento, archivo, con cambios, modo, activa)
        type Restored = (Document, Option<std::path::PathBuf>, bool, Mode, bool);
        let mut restored: Vec<Restored> = Vec::new();
        for entry in index["tabs"].as_array().into_iter().flatten() {
            let Some(file) = entry["file"].as_str() else {
                continue;
            };
            let Ok(document) = self.controller.store.open_document(&dir.join(file)) else {
                continue;
            };
            let path = entry["path"].as_str().map(std::path::PathBuf::from);
            let dirty = entry["dirty"].as_bool().unwrap_or(true);
            let mode = match entry["mode"].as_str() {
                Some("vector") => Mode::Vector,
                Some("animacion") => Mode::Animation,
                _ => Mode::Pixel,
            };
            let active = entry["active"].as_bool().unwrap_or(false);
            restored.push((document, path, dirty, mode, active));
        }
        if restored.is_empty() {
            return false;
        }
        let active = restored.iter().position(|r| r.4).unwrap_or(0);
        self.tabs.clear();
        for (i, (document, path, dirty, mode, _)) in restored.into_iter().enumerate() {
            if i == 0 {
                // La primera, en la sesión de la ventana.
                let _ = self.controller.swap_in(document, path.clone(), !dirty);
                let mut tab = DocTab::new(
                    self.next_tab_id,
                    self.main_session.clone(),
                    self.controller.saved_revision,
                );
                self.next_tab_id += 1;
                tab.path = path;
                tab.mode = mode;
                self.tabs.push(tab);
            } else {
                let j = self.push_tab(document, path, !dirty);
                self.tabs[j].mode = mode;
            }
        }
        self.unpark(active);
        true
    }

    /// Cambia el nombre del documento activo (F2). Si ya tiene archivo, el
    /// archivo también se renombra en su carpeta.
    fn rename_document(&mut self, name: &str) {
        let name = name.trim().trim_end_matches(".lume").trim();
        if name.is_empty() || name.contains(['/', '\\']) {
            self.shell.notice = Some("Ese nombre no vale.".into());
            return;
        }
        if let Some(path) = self.controller.path.clone() {
            let target = path.with_file_name(format!("{name}.lume"));
            if target != path {
                if target.exists() {
                    self.shell.notice = Some(format!("Ya hay un {name}.lume en esa carpeta."));
                    return;
                }
                if let Err(e) = std::fs::rename(&path, &target) {
                    self.shell.notice = Some(format!("No se pudo renombrar: {e}"));
                    return;
                }
                if let Some(recents) = &self.controller.recents {
                    let _ = recents.touch(&target, now_secs());
                }
                self.controller.path = Some(target);
            }
        }
        if let Err(e) = self
            .controller
            .runtime
            .block_on(self.controller.session.set_name(name.to_string()))
        {
            self.shell.notice = Some(e.to_string());
        }
    }

    /// Exporta una acción del esqueleto: cada fotograma es el dibujo posado
    /// (lo fijo y las piezas movidas) rasterizado; en fila (PNG) o GIF.
    fn export_rig_action(&mut self, name: &str, gif: bool, fps: u32, width: u32) {
        let Some(view) = self.shell.vec.view.clone() else {
            return;
        };
        let (Some(rig), Some(services)) = (view.result.rig.clone(), self.vec_services.clone())
        else {
            return;
        };
        let Some(action) = rig.actions.iter().find(|a| a.name == name).cloned() else {
            return;
        };
        let doc = self.controller.session.view().borrow().name.clone();
        let (kind, ext) = if gif {
            ("GIF animado", "gif")
        } else {
            ("Hoja de sprites PNG", "png")
        };
        let Some(path) = self
            .controller
            .dialogs
            .save_as(&format!("{doc}_{name}"), kind, ext)
        else {
            return;
        };
        let still = lume_domain::vec::studio::without_parts(&view.result.svg);
        let still = lume_domain::vec::studio::svg_inner(&still).to_string();
        let (w, h) = (view.result.width as f64, view.result.height as f64);
        let n = shell::vec_anim_frames(&action, fps);
        let span = if action.looping {
            n
        } else {
            n.saturating_sub(1).max(1)
        };
        let options = lume_app::ports::RasterOptions {
            width: Some(width.max(1)),
            ..Default::default()
        };
        let mut frames = Vec::with_capacity(n);
        for i in 0..n {
            let t = action.duration * i as f64 / span as f64;
            let svg = rig.posed_svg(Some(&action), t, &still);
            match services.rasterizer.rasterize(&svg, w, h, &options) {
                Ok(img) => frames.push(img),
                Err(e) => {
                    self.shell.notice = Some(format!("No se pudo pintar un fotograma: {e}"));
                    return;
                }
            }
        }
        let to_image = |img: &lume_app::ports::RasterImage| {
            let pixels = img
                .rgba
                .chunks_exact(4)
                .map(|p| lume_domain::Rgba::new(p[0], p[1], p[2], p[3]))
                .collect();
            lume_domain::anim::Image::from_pixels(img.width, img.height, pixels)
        };
        let bytes = if gif {
            let images: Option<Vec<_>> = frames.iter().map(to_image).collect();
            let durations = vec![(1000 / fps.max(1)).max(10); frames.len()];
            images
                .ok_or_else(|| "fotogramas de tamaños distintos".to_string())
                .and_then(|images| {
                    self.controller
                        .store
                        .encode_gif(&images, &durations, action.looping)
                        .map_err(|e| e.message)
                })
        } else {
            // En fila, de izquierda a derecha.
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
            encode_png(total, fh, &rgba).ok_or_else(|| "no se pudo codificar el PNG".to_string())
        };
        self.shell.notice = Some(
            match bytes.and_then(|b| std::fs::write(&path, b).map_err(|e| e.to_string())) {
                Ok(()) => format!("Exportado: {} ({n} fotogramas)", path.display()),
                Err(e) => format!("No se pudo exportar: {e}"),
            },
        );
    }

    /// Exporta el dibujo vectorial: PNG con el rasterizador; SVG, PDF e
    /// Illustrator con el exportador.
    fn export_vector(&mut self, format: intent::VecFormat, width: u32) {
        let (Some(services), Some(drawing)) = (
            self.vec_services.clone(),
            self.controller.session.view().borrow().vector.clone(),
        ) else {
            self.shell.notice = Some("No hay dibujo vectorial que exportar.".into());
            return;
        };
        let name = self.controller.session.view().borrow().name.clone();
        let Some(path) = self
            .controller
            .dialogs
            .save_as(&name, format.kind(), format.ext())
        else {
            return;
        };
        let result = services.engine.render(&drawing.source);
        let (w, h) = (result.width as f64, result.height as f64);
        let outcome: Result<(), String> = match format {
            intent::VecFormat::Png => services
                .rasterizer
                .rasterize(
                    &result.clean,
                    w,
                    h,
                    &lume_app::ports::RasterOptions {
                        width: Some(width),
                        ..Default::default()
                    },
                )
                .map_err(|e| e.to_string())
                .and_then(|image| {
                    encode_png(image.width, image.height, &image.rgba)
                        .ok_or_else(|| "no se pudo codificar el PNG".to_string())
                })
                .and_then(|png| std::fs::write(&path, png).map_err(|e| e.to_string())),
            _ => match &services.exporter {
                Some(exporter) => exporter
                    .export(&result.clean, &path)
                    .map(|_| ())
                    .map_err(|e| e.to_string()),
                None => Err("esta ventana no sabe exportar ese formato".into()),
            },
        };
        self.shell.notice = Some(match outcome {
            Ok(()) => format!("Exportado: {}", path.display()),
            Err(e) => format!("No se pudo exportar: {e}"),
        });
    }

    fn refresh_recents(&mut self) {
        self.recent_list = self
            .controller
            .recents
            .as_ref()
            .map(|r| r.list())
            .unwrap_or_default();
    }

    /// Inicio en una pestaña nueva (o en la activa si está vacía), para
    /// crear o abrir otro sin perder lo que hay abierto.
    fn open_home_tab(&mut self) {
        if self.viewing.is_some() {
            self.switch_to(None);
        }
        self.refresh_recents();
        self.screen = Screen::Studio;
        if self.tabs[self.active_tab].home {
            return;
        }
        if !self.tab_in_use() && !self.agent.running() {
            self.tabs[self.active_tab].home = true;
            return;
        }
        let blank = Document::new("sin título", 32, 32).expect("32×32 vale");
        let id = self.open_tab(blank, None, true, Mode::Pixel);
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
            tab.home = true;
        }
    }

    /// Cambia a la pestaña `i`. Si el agente de la que se deja está
    /// dibujando, sigue en segundo plano en su sesión.
    fn select_tab(&mut self, i: usize) {
        if self.viewing.is_some() {
            self.switch_to(None);
        }
        self.screen = Screen::Studio;
        if i == self.active_tab || i >= self.tabs.len() {
            return;
        }
        self.park_active();
        self.unpark(i);
    }

    /// La posición en la tira de lo que se ve (las pestañas propias y,
    /// detrás, los encargos).
    fn active_view(&self) -> usize {
        match self.viewing {
            Some(job) => self
                .job_tabs
                .iter()
                .position(|(j, _)| *j == job)
                .map(|p| self.tabs.len() + p)
                .unwrap_or(self.active_tab),
            None => self.active_tab,
        }
    }

    /// Enseña lo que hay en la posición `view` de la tira.
    fn select_view(&mut self, view: usize) {
        if view < self.tabs.len() {
            self.select_tab(view);
        } else if let Some(&(job, _)) = self.job_tabs.get(view - self.tabs.len()) {
            self.switch_to(Some(job));
        }
    }

    /// Cierra lo que hay en la posición `view` de la tira (un encargo solo
    /// deja de verse en la tira; sigue en «Encargos»).
    fn close_view(&mut self, view: usize, ask: bool) {
        if view < self.tabs.len() {
            self.close_tab(view, ask);
        } else if view - self.tabs.len() < self.job_tabs.len() {
            let (job, _) = self.job_tabs.remove(view - self.tabs.len());
            self.hidden_jobs.insert(job);
            if self.viewing == Some(job) {
                self.switch_to(None);
            }
        }
    }

    /// Cierra la pestaña `i` (si tiene cambios, primero se pregunta).
    fn close_tab(&mut self, i: usize, ask: bool) {
        if i >= self.tabs.len() {
            return;
        }
        if self.viewing.is_some() {
            self.switch_to(None);
        }
        let dirty = if i == self.active_tab {
            self.controller.is_dirty()
        } else {
            self.tabs[i].is_dirty()
        };
        if ask && dirty {
            self.closing_tab = Some(i);
            return;
        }
        if (i == self.active_tab && self.agent.running()) || self.tabs[i].busy() {
            self.shell.notice =
                Some("El agente está dibujando en esa pestaña: detenlo antes de cerrarla.".into());
            return;
        }
        if self.tabs.len() == 1 {
            // La última: queda Inicio en una pestaña nueva.
            let blank = Document::new("sin título", 32, 32).expect("32×32 vale");
            let j = self.push_tab(blank, None, true);
            self.tabs[j].home = true;
            self.unpark(j);
            self.tabs.remove(0);
            self.active_tab = 0;
            self.refresh_recents();
            self.screen = Screen::Studio;
            return;
        }
        if i == self.active_tab {
            let next = if i + 1 < self.tabs.len() {
                i + 1
            } else {
                i - 1
            };
            self.unpark(next);
        }
        self.tabs.remove(i);
        if self.active_tab > i {
            self.active_tab -= 1;
        }
    }

    /// Lo que la tira de pestañas necesita de cada una: las propias y,
    /// detrás, los encargos.
    fn tab_views(&self) -> Vec<shell::TabView> {
        let mut views: Vec<shell::TabView> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, t)| {
                if i == self.active_tab && self.viewing.is_none() {
                    shell::TabView {
                        id: t.id,
                        title: if t.home {
                            "Inicio".into()
                        } else {
                            self.controller.file_name()
                        },
                        dirty: self.controller.is_dirty() && !t.home,
                        mode: self.shell.mode,
                        busy: self.agent.running(),
                        home: t.home,
                        job: None,
                    }
                } else {
                    shell::TabView {
                        id: t.id,
                        title: t.title(),
                        dirty: !t.home && t.is_dirty(),
                        mode: t.mode,
                        busy: if i == self.active_tab {
                            self.agent.running()
                        } else {
                            t.busy()
                        },
                        home: t.home,
                        job: None,
                    }
                }
            })
            .collect();
        if let Some(art) = &self.art {
            let jobs = art.jobs();
            for &(job, id) in &self.job_tabs {
                let Some(status) = jobs.iter().find(|s| s.id == job) else {
                    continue;
                };
                let vector = art
                    .session(job)
                    .is_some_and(|s| s.view().borrow().vector.is_some());
                views.push(shell::TabView {
                    id,
                    title: status.name.clone(),
                    dirty: false,
                    mode: if vector { Mode::Vector } else { Mode::Pixel },
                    busy: status.state == lume_app::ArtState::Running,
                    home: false,
                    job: Some(job),
                });
            }
        }
        views
    }

    /// Tras guardar o descartar uno: pregunta por el siguiente o cierra.
    fn next_dirty_or_close(&mut self, ctx: &egui::Context) {
        if self.any_dirty() {
            self.focus_dirty();
        } else {
            self.asking_to_close = false;
            self.close_allowed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    /// Lo que llegó del agente: a la conversación y a la barra de estado.
    fn pump_agent(&mut self) {
        let resume = self.controller.settings.context == lume_app::ContextPolicy::Resume;
        for update in self.agent.poll() {
            // El agente de la ventana dibuja en el dibujo propio, aunque se
            // esté viendo un encargo.
            let panel = match self.viewing {
                None => &mut self.shell.agent,
                Some(_) => match self.stash.iter_mut().find(|d| d.job.is_none()) {
                    Some(doc) => &mut doc.panel,
                    None => continue,
                },
            };
            match update {
                agent_run::RunUpdate::Loop(event) => {
                    // Al terminar una vuelta, su imagen para la miniatura: el
                    // dibujo vectorial (en vector) o el lienzo.
                    let finished = matches!(event, lume_app::LoopEvent::TurnFinished { .. });
                    let own = self.agent.session().clone();
                    let vector = own
                        .view()
                        .borrow()
                        .vector
                        .clone()
                        .filter(|_| finished && self.shell.mode == Mode::Vector);
                    let image = match (finished, vector, self.vec_services.clone()) {
                        (false, _, _) => None,
                        (true, Some(drawing), Some(services)) => {
                            vector_thumbnail(&services, &drawing.source).map(Arc::new)
                        }
                        (true, _, _) => Some(Arc::new(canvas_image(&own.view().borrow().canvas))),
                    };
                    let panel = match self.viewing {
                        None => &mut self.shell.agent,
                        Some(_) => match self.stash.iter_mut().find(|d| d.job.is_none()) {
                            Some(doc) => &mut doc.panel,
                            None => continue,
                        },
                    };
                    panel.apply_loop(&event, image, resume);
                }
                agent_run::RunUpdate::Finished(summary) => {
                    let error = matches!(summary.end, lume_app::LoopEnd::Failed(_));
                    panel.finished(summary.reason(), error);
                    // Terminó la pasada de correcciones: fuera las hechas
                    // (si falló o se detuvo, vuelven a esperar en la cola).
                    if self.viewing.is_none() {
                        let undone = error || matches!(summary.end, lume_app::LoopEnd::Cancelled);
                        self.shell.vec.corrections_finished(undone);
                    }
                }
            }
        }
        // Los agentes de las pestañas que no se ven: su conversación sigue en
        // su panel.
        for tab in &mut self.tabs {
            let Some(agent) = tab.agent.as_mut() else {
                continue;
            };
            let mut done = false;
            for update in agent.sup.poll() {
                match update {
                    agent_run::RunUpdate::Loop(event) => {
                        tab.panel.apply_loop(&event, None, resume);
                    }
                    agent_run::RunUpdate::Finished(summary) => {
                        let error = matches!(summary.end, lume_app::LoopEnd::Failed(_));
                        tab.panel.finished(summary.reason(), error);
                        let undone = error || matches!(summary.end, lume_app::LoopEnd::Cancelled);
                        shell::finish_corrections(&mut tab.corrections, undone);
                        done = true;
                    }
                }
            }
            if done {
                tab.agent = None;
            }
        }
        let running = self.agent.running() && self.viewing.is_none();
        let vector_doc = self.controller.session.view().borrow().vector.is_some();
        self.shell.agent_view = agent_panel::AgentView {
            animation: self.shell.mode == Mode::Animation && !vector_doc,
            vector: self.shell.mode == Mode::Vector,
            vector_anim: self.shell.mode == Mode::Animation && vector_doc,
            settings: self.controller.settings.clone(),
            running,
            preview_version: self.shell.preview.as_ref().map(|p| p.number),
        };
        let current = self
            .shell
            .agent
            .loop_box
            .as_ref()
            .and_then(|lb| lb.current.map(|n| (n, lb.of)));
        let fixing = self.shell.vec.corrections.iter().filter(|c| c.sent).count();
        let fixing = (fixing > 0 && self.shell.mode == Mode::Vector).then_some(fixing);
        self.shell.agent_status = running.then(|| match (fixing, current) {
            (Some(1), _) => "Haciendo la corrección".to_string(),
            (Some(n), _) => format!("Haciendo {n} correcciones"),
            (None, Some((n, of))) => format!("Vuelta {n} de {of}"),
            (None, None) => "Dibujando…".to_string(),
        });
        self.home.provider = agent_panel::provider_name(self.controller.settings.provider).into();
    }

    fn dispatch(&mut self, intents: Vec<Intent>) {
        for intent in intents {
            if intent == Intent::Export {
                let vector_doc = self.controller.session.view().borrow().vector.is_some();
                if self.shell.mode == Mode::Animation && vector_doc {
                    self.shell.vanim.export = Some((false, 12, 256));
                } else if self.shell.mode == Mode::Animation {
                    self.shell.anim.open_export();
                } else if self.shell.mode == Mode::Vector {
                    self.shell.vec.export = Some((intent::VecFormat::Png, 1024));
                } else {
                    self.shell.open_export_dialog();
                }
                continue;
            }
            // Lo que cambia de documento se hace en el dibujo propio.
            if self.viewing.is_some()
                && matches!(
                    intent,
                    Intent::NewDocument { .. }
                        | Intent::OpenFile
                        | Intent::OpenPath(_)
                        | Intent::OpenRecent(_)
                        | Intent::ImportPxcli
                        | Intent::ImportPxcliFrom(_)
                        | Intent::Describe { .. }
                )
            {
                self.switch_to(None);
            }
            match intent {
                Intent::OpenJob(id) => {
                    self.switch_to(Some(id));
                    continue;
                }
                Intent::OpenMain => {
                    self.switch_to(None);
                    continue;
                }
                Intent::AgentDraft(text) => {
                    self.shell.agent.draft = text;
                    continue;
                }
                Intent::RequestVariants { prompt } => {
                    self.request_variants(&prompt);
                    continue;
                }
                Intent::ChooseVariant(index) => {
                    self.choose_variant(index);
                    continue;
                }
                Intent::MixVariants(text) => {
                    let (Some(variants), Ok(document)) = (
                        self.variants.clone(),
                        self.controller
                            .runtime
                            .block_on(self.controller.session.snapshot()),
                    ) else {
                        continue;
                    };
                    if let Err(err) = self
                        .controller
                        .runtime
                        .block_on(variants.mix(&document, &text))
                    {
                        self.shell.notice = Some(format!("No se pudo mezclar: {err}"));
                    }
                    continue;
                }
                Intent::ShowVariants(show) => {
                    self.shell.anim.showing_variants = show;
                    continue;
                }
                Intent::VectorZone => {
                    self.vector_zone();
                    continue;
                }
                Intent::VectorSelect => {
                    self.vector_select();
                    continue;
                }
                Intent::RunCorrections => {
                    self.run_corrections();
                    continue;
                }
                Intent::ReplayVector => {
                    self.replay_vector();
                    continue;
                }
                Intent::ExportAnimation {
                    format,
                    scale,
                    file_name,
                    open_folder,
                } => {
                    self.export_animation(format, scale, &file_name, open_folder);
                    continue;
                }
                Intent::AgentSend { prompt, .. } if self.viewing.is_some() => {
                    let (Some(id), Some(art)) = (self.viewing, self.art.clone()) else {
                        continue;
                    };
                    // En marcha: a la vuelta siguiente. Terminado: otra
                    // ejecución del encargo con estos cambios.
                    if art.send_message(id, &prompt) {
                        self.shell.agent.message_for_next_turn(&prompt);
                    } else if let Err(e) = self
                        .controller
                        .runtime
                        .block_on(art.continue_job(id, &prompt))
                    {
                        self.shell.notice = Some(format!("No se pudo seguir el encargo: {e}"));
                    }
                    continue;
                }
                Intent::AgentStop if self.viewing.is_some() => {
                    if let (Some(id), Some(art)) = (self.viewing, &self.art) {
                        art.cancel(id);
                    }
                    continue;
                }
                Intent::AgentSend {
                    prompt,
                    attachments,
                    continuing,
                } => {
                    // Con el bucle en marcha, el mensaje va a la vuelta
                    // siguiente sin detenerlo.
                    if self.agent.send_message(&prompt) {
                        self.shell.agent.message_for_next_turn(&prompt);
                    } else {
                        self.start_agent(&prompt, attachments, continuing);
                    }
                    continue;
                }
                Intent::ShowVersion(number) => {
                    self.show_version(number);
                    continue;
                }
                Intent::RestoreVersion(number) => {
                    let restored = self.controller.runtime.block_on(
                        self.controller
                            .session
                            .restore_version(number, lume_domain::Origin::User),
                    );
                    self.shell.preview = None;
                    self.shell.notice = Some(match restored {
                        Ok(()) => format!("De vuelta a la versión {number}"),
                        Err(err) => format!("No se pudo volver a la versión {number}: {err}"),
                    });
                    continue;
                }
                Intent::ClosePreview => {
                    self.shell.preview = None;
                    continue;
                }
                // Viendo una versión no se dibuja ni se deshace.
                Intent::Apply { .. } | Intent::Undo | Intent::Redo | Intent::UndoSteps(_)
                    if self.shell.preview.is_some() =>
                {
                    continue;
                }
                Intent::Describe { prompt } => {
                    self.start_agent(&prompt, Vec::new(), false);
                    continue;
                }
                Intent::AgentStop => {
                    self.agent.stop();
                    continue;
                }
                Intent::AgentAttach => {
                    self.attach_reference();
                    continue;
                }
                Intent::AgentPasteImage(attachment) => {
                    self.add_reference(attachment);
                    continue;
                }
                // Desde Inicio («Nuevo dibujo vectorial»…): el modo del lienzo
                // nuevo. (El selector de la cabecera ya lo ha cambiado.)
                Intent::SwitchMode(mode) => {
                    self.shell.mode = mode;
                }
                Intent::SelectTab(i) => {
                    self.select_view(i);
                    continue;
                }
                Intent::ExportVector { format, width } => {
                    self.export_vector(format, width);
                    continue;
                }
                Intent::ExportRigAction {
                    action,
                    gif,
                    fps,
                    width,
                } => {
                    self.export_rig_action(&action, gif, fps, width);
                    continue;
                }
                Intent::RenameDocument(name) => {
                    self.rename_document(&name);
                    continue;
                }
                Intent::CloseTab(i) => {
                    self.close_view(i, true);
                    continue;
                }
                _ => {}
            }
            // Inicio se abre en una pestaña: lo abierto sigue en las suyas.
            if intent == Intent::GoHome && self.screen != Screen::Settings {
                self.open_home_tab();
                continue;
            }
            if intent == Intent::CloseSettings {
                self.settings_open = false;
                self.settings_screen.notice = None;
                continue;
            }
            // Un documento nuevo o abierto va a una pestaña nueva (salvo que
            // la actual esté vacía). Si no llega a abrirse (diálogo
            // cancelado, error), se vuelve a la de antes.
            let opens = matches!(
                intent,
                Intent::NewDocument { .. }
                    | Intent::OpenFile
                    | Intent::OpenPath(_)
                    | Intent::OpenRecent(_)
                    | Intent::ImportPxcli
                    | Intent::ImportPxcliFrom(_)
            );
            let mut previous_tab = None;
            if opens
                && self.viewing.is_none()
                && (self.tab_in_use() || self.agent.running())
                && self.park_active()
            {
                // Una pestaña nueva, con su sesión. La que se deja sigue
                // con su agente si dibuja.
                previous_tab = Some(self.active_tab);
                let blank = Document::new("sin título", 32, 32).expect("32×32 vale");
                self.open_tab(blank, None, true, Mode::Pixel);
            }
            let revision = self.controller.session.view().borrow().revision;
            let session_before = self.controller.session.clone();
            let outcome = self.controller.handle(intent);
            let opened = self.controller.session.view().borrow().revision != revision
                || !session_before.same(&self.controller.session);
            if let Some(previous) = previous_tab
                && !opened
            {
                // No se abrió nada: fuera la pestaña nueva y vuelve la de antes.
                self.unpark(previous);
                self.tabs.pop();
            } else if opens
                && opened
                && let Some(tab) = self.tabs.get_mut(self.active_tab)
            {
                // Inicio de esta pestaña deja paso al documento.
                tab.home = false;
            }
            if self.settings_open
                && let Some(message) = outcome.error.clone().or(outcome.notice.clone())
            {
                self.settings_screen.notice = Some(message);
            }
            if let Some(screen) = outcome.screen {
                match screen {
                    // Ajustes se abre en su propia ventana: la principal
                    // sigue como estaba.
                    Screen::Settings => {
                        self.settings_open = true;
                        self.settings_screen.notice = None;
                        ctx_focus_settings(&self.ctx);
                        continue;
                    }
                    Screen::Studio => {
                        self.shell.canvas.fit_pending = true;
                        self.shell.selection = None;
                        self.reset_vector();
                    }
                    Screen::Home => {
                        self.recent_list = self
                            .controller
                            .recents
                            .as_ref()
                            .map(|r| r.list())
                            .unwrap_or_default();
                    }
                }
                self.screen = screen;
            }
            if !self.settings_open {
                self.home.error = outcome.error.clone();
                if let Some(message) = outcome.error.or(outcome.notice) {
                    self.shell.notice = Some(message);
                }
            }
        }
    }

    /// Cerrar una pestaña con cambios: guardar, descartar o no cerrarla.
    fn guard_tab_close(&mut self, ctx: &egui::Context) {
        let Some(i) = self.closing_tab else {
            return;
        };
        let name = match i == self.active_tab {
            true => self.controller.file_name(),
            false => self.tabs.get(i).map(DocTab::title).unwrap_or_default(),
        };
        match unsaved_dialog(ctx, &name) {
            Some(UnsavedChoice::Save) => {
                self.closing_tab = None;
                self.select_tab(i);
                if self.active_tab == i {
                    self.dispatch(vec![Intent::Save]);
                    if !self.controller.is_dirty() {
                        self.close_tab(i, false);
                    }
                }
            }
            Some(UnsavedChoice::Discard) => {
                self.closing_tab = None;
                self.close_tab(i, false);
            }
            Some(UnsavedChoice::Cancel) => self.closing_tab = None,
            None => {}
        }
    }

    /// Cierre: si hay cambios sin guardar, pregunta antes.
    fn guard_close(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        let stop_requested = self.stop.is_cancelled() && !self.stop_handled;
        if stop_requested {
            self.stop_handled = true;
        }
        if close_requested || stop_requested {
            self.autosave_now();
        }
        if (close_requested || stop_requested) && !self.close_allowed {
            if self.any_dirty() {
                if close_requested {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                }
                self.focus_dirty();
                self.asking_to_close = true;
            } else {
                self.close_allowed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if self.asking_to_close {
            match unsaved_dialog(ctx, &self.controller.file_name()) {
                Some(UnsavedChoice::Save) => {
                    self.dispatch(vec![Intent::Save]);
                    if self.controller.is_dirty() {
                        // No se guardó (cancelado o error): no se cierra.
                        self.asking_to_close = false;
                    } else {
                        self.next_dirty_or_close(ctx);
                    }
                }
                Some(UnsavedChoice::Discard) => {
                    // Este se descarta; quedan los demás.
                    self.controller.saved_revision =
                        Some(self.controller.session.view().borrow().revision);
                    self.next_dirty_or_close(ctx);
                }
                Some(UnsavedChoice::Cancel) => self.asking_to_close = false,
                None => {}
            }
        }
    }
}

impl eframe::App for StudioApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::from(theme::BG).to_array()
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.pump_agent();
        self.pump_jobs();
        self.pump_anim();
        self.pump_vec();
        self.pump_window();
        self.pump_variants();
        self.shell.file_name = Some(self.controller.file_name());
        self.shell.dirty = self.controller.is_dirty();
        self.shell.show_commands = self.controller.settings.show_commands;
        self.shell.tabs = self.tab_views();
        // Los avisos de la barra de estado se van a los 6 segundos.
        let now = ui.input(|i| i.time);
        // Autoguardado frecuente de todas las pestañas (cada 10 s; si la
        // ventana está quieta, se despierta justo para eso).
        const AUTOSAVE_EVERY: f64 = 10.0;
        if now - self.last_autosave >= AUTOSAVE_EVERY {
            self.last_autosave = now;
            self.autosave_now();
        }
        let left = (AUTOSAVE_EVERY - (now - self.last_autosave)).max(0.5);
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f64(left));
        match (&self.shell.notice, &self.notice_since) {
            (None, _) => self.notice_since = None,
            (Some(text), Some((seen, since))) if text == seen => {
                if now - since > 6.0 {
                    self.shell.notice = None;
                    self.notice_since = None;
                } else {
                    ui.ctx()
                        .request_repaint_after(std::time::Duration::from_secs(1));
                }
            }
            (Some(text), _) => {
                self.notice_since = Some((text.clone(), now));
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs(1));
            }
        }
        self.shell.active_tab = self.active_view();
        self.publish_tabs();
        self.shell.canvas.speed = self.controller.settings.draw_speed;
        // Miniaturas del historial que no se vieron nacer (al abrir un
        // archivo): se sacan del documento, una vez por revisión.
        if self.screen == Screen::Studio
            && self.shell.agent.tab == agent_panel::PanelTab::History
            && self.shell.history_thumbs.needs_document()
            && let Ok(document) = self
                .controller
                .runtime
                .block_on(self.controller.session.snapshot())
        {
            self.shell.history_thumbs.fill(&document);
        }
        let intents = match self.screen {
            Screen::Home => self.home.show(ui, &self.recent_list, now_secs()),
            Screen::Studio
                if self.viewing.is_none()
                    && self.tabs.get(self.active_tab).is_some_and(|t| t.home) =>
            {
                // Inicio dentro de su pestaña, con la tira de pestañas encima.
                let shell = &mut self.shell;
                let mut strip = Vec::new();
                let mut out = self.home.show_with_tabs(
                    ui,
                    &self.recent_list,
                    now_secs(),
                    Some(&mut |ui: &mut egui::Ui| shell.tab_strip(ui, &mut strip)),
                );
                out.extend(strip);
                out
            }
            Screen::Studio => {
                let mut vm = self.controller.session.view().borrow().clone();
                if let Some(preview) = &self.shell.preview {
                    // La versión en lugar del dibujo, con una revisión propia
                    // para que el lienzo rehaga la textura.
                    let mut shown = (*vm).clone();
                    shown.canvas = preview.canvas.clone();
                    shown.revision = PREVIEW_REVISION + u64::from(preview.number);
                    shown.last_change = None;
                    vm = Arc::new(shown);
                }
                self.shell.show(ui, &vm)
            }
            Screen::Settings => {
                // Se pidió empezar en Ajustes: Inicio detrás y Ajustes en su
                // ventana.
                self.screen = Screen::Home;
                self.settings_open = true;
                Vec::new()
            }
        };
        self.dispatch(intents);
        if self.settings_open {
            let builder = egui::ViewportBuilder::default()
                .with_title("Ajustes · LumeArt")
                .with_app_id("lumeart")
                .with_inner_size([1300.0, 820.0])
                .with_min_inner_size([900.0, 560.0])
                .with_icon(shared_icon());
            let intents = ctx.show_viewport_immediate(
                egui::ViewportId::from_hash_of("lumeart-ajustes"),
                builder,
                |ui, _class| {
                    let mut out = self.settings_screen.show(
                        ui,
                        &self.controller.settings,
                        &self.controller.keys,
                    );
                    if ui.input(|i| i.viewport().close_requested()) {
                        out.push(Intent::CloseSettings);
                    }
                    out
                },
            );
            self.dispatch(intents);
        }
        self.guard_tab_close(&ctx);
        self.guard_close(&ctx);
    }
}

/// Todo lo que necesita la ventana.
pub struct StudioOptions {
    pub session: SessionHandle,
    /// Se cancela cuando piden parar (`lumeart stop`): la ventana se cierra
    /// (preguntando antes si hay cambios sin guardar).
    pub stop: CancellationToken,
    /// Runtime de la sesión.
    pub runtime: tokio::runtime::Handle,
    pub store: Arc<dyn DocumentStore>,
    pub recents: Option<Arc<dyn RecentStore>>,
    pub dialogs: Arc<dyn FileDialogs>,
    /// Ajustes (`config.toml`) y llavero de las claves de API.
    pub settings: Arc<dyn SettingsStore>,
    pub secrets: Arc<dyn SecretStore>,
    /// Ruta de `claude`, si está instalado (se muestra en Ajustes).
    pub claude_path: Option<std::path::PathBuf>,
    /// Prepara el agente de cada ejecución (`None`: sin agente).
    pub agent: Option<Arc<dyn AgentBackend>>,
    /// Dirección de la API local, para la barra de estado.
    pub server: Option<String>,
    /// Pantalla inicial.
    pub screen: Screen,
    /// Encargos que llegan por MCP (`None`: esta ventana no los acepta).
    pub art: Option<Arc<LocalArt>>,
    /// Motor de `anim.js` para el modo animación (`None`: sin animación).
    pub anim_engine: Option<Arc<dyn AnimEngine>>,
    /// Agentes para las variantes de una animación (`None`: sin variantes).
    pub variant_agents: Option<Arc<dyn lume_app::ArtAgents>>,
    /// Motor de `dibujo.js`, rasterizador y buscador de formas del modo
    /// vector (`None`: sin modo vector).
    pub vector: Option<VecServices>,
    /// Carpeta de autoguardado de las pestañas: se guardan cada poco y se
    /// restauran al abrir la ventana (como Sublime o Photoshop).
    pub autosave: Option<std::path::PathBuf>,
    /// Peticiones de otros agentes (por MCP) a esta ventana.
    pub window: Option<lume_app::studio::WindowChannel>,
}

/// Lo que necesita el modo vector.
#[derive(Clone)]
pub struct VecServices {
    pub engine: Arc<dyn lume_app::ports::VecEngine>,
    pub rasterizer: Arc<dyn lume_app::ports::Rasterizer>,
    pub shapes: Arc<dyn lume_app::ports::ShapeFinder>,
    /// SVG, PDF e Illustrator (el PNG lo hace el rasterizador).
    pub exporter: Option<Arc<dyn lume_app::ports::VectorExporter>>,
}

/// Pasos de «Repetir trazado» y pausa entre ellos.
const REPLAY_STEPS: usize = 24;
const REPLAY_PAUSE_MS: u64 = 60;

/// Cómo se rasteriza el dibujo para la pantalla: de 512 a 1600 píxeles de
/// ancho (la rejilla la dibuja la ventana encima).
fn raster_options(width: f64, _height: f64) -> lume_app::ports::RasterOptions {
    lume_app::ports::RasterOptions {
        width: Some(width.clamp(512.0, 1600.0) as u32),
        ..Default::default()
    }
}

/// Ejecuta `dibujo.js` y pinta el resultado entero.
fn vec_view(services: &VecServices, source: String, key: u64) -> shell::VecView {
    let result = services.engine.render(&source);
    let layers = lume_domain::vec::studio::layers(&result.clean);
    let (w, h) = (result.width as f64, result.height as f64);
    let image = if w <= 0.0 || h <= 0.0 {
        Err("dibujo.js no fija el tamaño con canvas()".to_string())
    } else {
        services
            .rasterizer
            .rasterize(&result.clean, w, h, &raster_options(w, h))
            .map_err(|e| e.to_string())
    };
    shell::VecView {
        source,
        result,
        layers,
        image,
        key,
    }
}

/// Un lienzo de píxeles como imagen de egui.
fn canvas_image(canvas: &lume_domain::Canvas) -> egui::ColorImage {
    egui::ColorImage::from_rgba_unmultiplied(
        [canvas.width() as usize, canvas.height() as usize],
        &canvas.to_rgba_bytes(),
    )
}

/// Miniatura de un dibujo vectorial (160 px de ancho).
fn vector_thumbnail(services: &VecServices, source: &str) -> Option<egui::ColorImage> {
    let result = services.engine.render(source);
    let (w, h) = (result.width as f64, result.height as f64);
    if w <= 0.0 || h <= 0.0 {
        return None;
    }
    let options = lume_app::ports::RasterOptions {
        width: Some(160),
        ..Default::default()
    };
    let image = services
        .rasterizer
        .rasterize(&result.clean, w, h, &options)
        .ok()?;
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [image.width as usize, image.height as usize],
        &image.rgba,
    ))
}

/// Lo que se pide al agente antes del encargo cuando anima un dibujo
/// vectorial: cómo es el esqueleto de `dibujo.js` y cómo comprobarlo.
const VECTOR_ANIM_GOAL: &str = "Animación del dibujo vectorial (en dibujo.js, con su esqueleto). No cambies cómo se ve el dibujo en reposo, salvo para dejar sitio en el lienzo: lo que se mueve no debe salirse de él en ningún fotograma. Si aún no tiene esqueleto: pon cada parte que se mueve dentro de part(hueso, () => { ... }) y declara antes sus huesos con bone(nombre, { parent, at: [x, y] }), con `at` en la articulación sobre la que gira (la bisagra de una tapa, un hombro); lo que no se mueve puede ir en una pieza del hueso raíz o fuera de las piezas. Las acciones: action(nombre, { duration: segundos, loop: true o false }, [[0, { hueso: grados }], [0.5, { ... }], [1, { ... }]]) con t de 0 a 1, grados en el sentido de las agujas del reloj y { rot, x, y, scale, alpha } para además desplazar el hueso, estirarlo o aplastarlo alrededor de su articulación (scale: número o [sx, sy]; un ojo que pestañea se aplasta en vertical) o desvanecerlo (alpha de 0 a 1). Compruébalo con `lumeart vec check dibujo.js` y mira cada acción fotograma a fotograma con `lumeart vec poses dibujo.js -o poses.png` (ábrela con Read). Lo que pide el usuario: ";

/// Las piezas del esqueleto, cada una recortada a su caja, y lo que queda
/// fijo, pintados a unos 1400 px de ancho de dibujo.
fn rig_images(services: &VecServices, view: &shell::VecView) -> shell::RigImages {
    let rig = view.result.rig.clone().unwrap_or_default();
    let (w, h) = (view.result.width as f64, view.result.height as f64);
    let scale = (1400.0 / w.max(1.0)).min(2.0);
    let parts = rig
        .parts
        .iter()
        .map(|part| {
            let [x, y, bw, bh] = part.bbox;
            let options = lume_app::ports::RasterOptions {
                width: Some(((bw * scale).round() as u32).max(1)),
                crop: Some([x, y, bw.max(1.0), bh.max(1.0)]),
                ..Default::default()
            };
            services
                .rasterizer
                .rasterize(&rig.part_svg(part), w, h, &options)
                .ok()
        })
        .collect();
    let backdrop = services
        .rasterizer
        .rasterize(
            &lume_domain::vec::studio::without_parts(&view.result.svg),
            w,
            h,
            &lume_app::ports::RasterOptions {
                width: Some(((w * scale).round() as u32).max(1)),
                ..Default::default()
            },
        )
        .ok();
    shell::RigImages {
        key: view.key,
        parts,
        backdrop,
    }
}

/// El dibujo pintado en trozos (cada capa sola y lo de entre capas), del
/// mismo tamaño que la imagen entera: superpuestos dan el dibujo.
fn slice_images(services: &VecServices, view: &shell::VecView) -> Vec<shell::SliceImage> {
    let (w, h) = (view.result.width as f64, view.result.height as f64);
    let options = raster_options(w, h);
    lume_domain::vec::studio::slices(&view.result.clean)
        .into_iter()
        .filter_map(|slice| {
            let image = services
                .rasterizer
                .rasterize(&slice.svg, w, h, &options)
                .ok()?;
            Some(shell::SliceImage {
                layer: slice.layer,
                image,
            })
        })
        .collect()
}

/// Abre la ventana y bloquea hasta que se cierra (o hasta que se cancele
/// `stop`). Debe llamarse desde el hilo principal.
pub fn run_studio(options: StudioOptions) -> eframe::Result {
    eframe::run_native(
        WINDOW_TITLE,
        native_options(),
        Box::new(move |cc| Ok(Box::new(StudioApp::new(cc, options)))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_icon_is_the_brand_icon() {
        let icon = window_icon();
        let n = WINDOW_ICON_SIZE;
        assert_eq!((icon.width, icon.height), (n, n));
        assert_eq!(icon.rgba.len(), (n * n * 4) as usize);
        // Las esquinas son transparentes (cuadrado redondeado) y el centro no.
        assert_eq!(icon.rgba[3], 0);
        let center = ((n / 2 * n + n / 2) * 4) as usize;
        assert_eq!(icon.rgba[center + 3], 255);
        // En X11, el icono (ancho, alto y un entero por píxel) cabe en una
        // petición sin BIG-REQUESTS (262 140 bytes).
        if cfg!(target_os = "linux") {
            assert!((2 + n * n) * 4 <= 262_140);
        }
    }

    #[test]
    fn native_options_have_title_and_size() {
        let options = native_options();
        assert_eq!(options.viewport.title.as_deref(), Some("LumeArt"));
        assert_eq!(options.viewport.inner_size, Some(egui::vec2(1440.0, 900.0)));
        assert!(options.viewport.icon.is_some());
    }

    /// El controlador con una sesión de verdad: crear un lienzo y abrir un
    /// reciente cambian el documento y llevan al studio.
    #[test]
    fn controller_opens_and_creates_documents() {
        use lume_app::testing::{MemoryRecents, MemoryStore};
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let store = Arc::new(MemoryStore::new());
        let recents = Arc::new(MemoryRecents::default());
        let session = {
            let _guard = runtime.enter();
            lume_app::Session::spawn(Document::new("inicio", 16, 16).unwrap(), store.clone())
        };
        let mut controller = AppController::new(
            session.handle.clone(),
            runtime.handle().clone(),
            store.clone(),
            Some(recents.clone()),
            Arc::new(FakeDialogs::default()),
        );

        let outcome = controller.handle(Intent::NewDocument {
            width: 32,
            height: 32,
            palette: Some("pico-8".into()),
        });
        assert_eq!(outcome.screen, Some(Screen::Studio));
        let vm = session.handle.view().borrow().clone();
        assert_eq!((vm.canvas.width(), vm.canvas.height()), (32, 32));
        assert_eq!(vm.palette.len(), 16, "paleta PICO-8");

        let mut saved = Document::new("pocion", 8, 8).unwrap();
        saved
            .apply(
                lume_domain::Command::SetPixel {
                    x: 1,
                    y: 1,
                    color: lume_domain::Rgba::rgb(1, 2, 3),
                },
                Origin::User,
            )
            .unwrap();
        lume_app::DocumentStore::save_document(
            &*store,
            &saved,
            std::path::Path::new("/arte/pocion.lume"),
        )
        .unwrap();
        let outcome = controller.handle(Intent::OpenRecent("/arte/pocion.lume".into()));
        assert_eq!(outcome.screen, Some(Screen::Studio));
        assert_eq!(outcome.error, None);
        let vm = session.handle.view().borrow().clone();
        assert_eq!(vm.name, "pocion");
        assert_eq!(
            vm.canvas.get(1, 1).unwrap(),
            lume_domain::Rgba::rgb(1, 2, 3)
        );
        assert_eq!(
            recents.touched.lock().unwrap()[0].0,
            std::path::PathBuf::from("/arte/pocion.lume")
        );

        let outcome = controller.handle(Intent::OpenRecent("/no/esta.lume".into()));
        assert_eq!(outcome.screen, None);
        assert!(
            outcome
                .error
                .unwrap()
                .starts_with("No se pudo abrir /no/esta.lume")
        );
        runtime.block_on(session.close());
    }

    /// Controlador de prueba con almacén, recientes y diálogos en memoria.
    fn controller_with_doubles() -> (
        tokio::runtime::Runtime,
        lume_app::Session,
        AppController,
        Arc<lume_app::testing::MemoryStore>,
        Arc<FakeDialogs>,
    ) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let store = Arc::new(lume_app::testing::MemoryStore::new());
        let session = {
            let _guard = runtime.enter();
            lume_app::Session::spawn(Document::new("sin título", 8, 8).unwrap(), store.clone())
        };
        let dialogs = Arc::new(FakeDialogs::default());
        let controller = AppController::new(
            session.handle.clone(),
            runtime.handle().clone(),
            store.clone(),
            Some(Arc::new(lume_app::testing::MemoryRecents::default())),
            dialogs.clone(),
        );
        (runtime, session, controller, store, dialogs)
    }

    fn draw(runtime: &tokio::runtime::Runtime, session: &lume_app::Session, line: &str) {
        runtime
            .block_on(session.handle.line(line, Origin::User, None))
            .unwrap();
    }

    #[test]
    fn save_marks_clean_and_save_as_uses_the_dialog() {
        let (runtime, session, mut c, store, dialogs) = controller_with_doubles();
        assert!(!c.is_dirty(), "un documento recién abierto está limpio");
        assert_eq!(c.file_name(), "sin título.lume");
        draw(&runtime, &session, "set_pixel 1 1 red");
        assert!(c.is_dirty());

        // Sin archivo, Guardar pregunta dónde (y si se cancela, no pasa nada).
        let outcome = c.handle(Intent::Save);
        assert_eq!(outcome, Outcome::default());
        assert!(c.is_dirty());
        *dialogs.save.lock().unwrap() = Some("/juego/arte/cofre.lume".into());
        let outcome = c.handle(Intent::Save);
        assert_eq!(
            outcome.notice.as_deref(),
            Some("Guardado en /juego/arte/cofre.lume")
        );
        assert!(!c.is_dirty());
        assert_eq!(c.file_name(), "cofre.lume");
        assert_eq!(session.handle.view().borrow().name, "cofre");
        assert_eq!(
            dialogs.suggested.lock().unwrap().as_slice(),
            ["sin título".to_string(), "sin título".to_string()]
        );

        // Con archivo, Guardar no pregunta.
        draw(&runtime, &session, "set_pixel 2 2 blue");
        assert!(c.is_dirty());
        *dialogs.save.lock().unwrap() = None;
        c.handle(Intent::Save);
        assert!(!c.is_dirty());
        let saved = store
            .open_document(std::path::Path::new("/juego/arte/cofre.lume"))
            .unwrap();
        assert_eq!(saved.name, "cofre");
        assert_eq!(
            saved.canvas().get(2, 2).unwrap(),
            lume_domain::Rgba::rgb(0, 0, 255)
        );

        // Abrir otro y volver: el contenido es el guardado y está limpio.
        c.handle(Intent::NewDocument {
            width: 4,
            height: 4,
            palette: None,
        });
        let outcome = c.handle(Intent::OpenPath("/juego/arte/cofre.lume".into()));
        assert_eq!(outcome.screen, Some(Screen::Studio));
        assert!(!c.is_dirty());
        assert_eq!(*session.handle.view().borrow().canvas, *saved.canvas());
        runtime.block_on(session.close());
    }

    #[test]
    fn export_uses_the_chosen_scale() {
        let (runtime, session, mut c, store, dialogs) = controller_with_doubles();
        draw(&runtime, &session, "fill_rect 0 0 2 2 red");
        let outcome = c.handle(Intent::ExportPngTo {
            path: "/juego/a.png".into(),
            scale: 4,
        });
        assert_eq!(
            outcome.notice.as_deref(),
            Some("Exportado a /juego/a.png (4×)")
        );
        *dialogs.png.lock().unwrap() = Some("/juego/b.png".into());
        c.handle(Intent::ExportPng { scale: 16 });
        let exports = store.exports();
        assert_eq!(exports.len(), 2);
        assert_eq!((exports[0].scale, exports[1].scale), (4, 16));
        assert_eq!(exports[1].path, std::path::PathBuf::from("/juego/b.png"));
        assert_eq!(
            exports[0].canvas.get(1, 1).unwrap(),
            lume_domain::Rgba::rgb(255, 0, 0)
        );
        // Exportar no cambia el estado de guardado.
        assert!(c.is_dirty());
        let outcome = c.handle(Intent::ExportPngTo {
            path: "/juego/c.png".into(),
            scale: 0,
        });
        assert!(
            outcome
                .error
                .unwrap()
                .contains("scale must be between 1 and 64")
        );
        runtime.block_on(session.close());
    }

    #[test]
    fn importing_a_pxcli_folder_leaves_it_unsaved() {
        let (runtime, session, mut c, store, dialogs) = controller_with_doubles();
        let mut run = Document::new("20261028-reloj", 16, 16).unwrap();
        run.apply(
            lume_domain::Command::SetPixel {
                x: 3,
                y: 3,
                color: lume_domain::Rgba::rgb(9, 9, 9),
            },
            Origin::Cli,
        )
        .unwrap();
        store.put_pxcli_run("/runs/reloj", run);
        *dialogs.folder.lock().unwrap() = Some("/runs/reloj".into());
        let outcome = c.handle(Intent::ImportPxcli);
        assert_eq!(outcome.screen, Some(Screen::Studio));
        assert!(c.is_dirty(), "lo importado aún no está guardado");
        assert_eq!(c.path, None);
        assert_eq!(
            session.handle.view().borrow().canvas.get(3, 3).unwrap(),
            lume_domain::Rgba::rgb(9, 9, 9)
        );
        let outcome = c.handle(Intent::ImportPxcliFrom("/runs/nada".into()));
        assert!(
            outcome
                .error
                .unwrap()
                .starts_with("No se pudo importar /runs/nada")
        );
        runtime.block_on(session.close());
    }
}
