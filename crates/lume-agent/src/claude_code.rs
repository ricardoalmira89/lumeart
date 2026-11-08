//! `ClaudeCodeRunner`: el agente con la suscripción de Claude Code del
//! usuario. Lanza `claude -p` como en `internal/web/agent.go` de pxcli:
//!
//! - Herramientas `Bash` (solo `lumeart …`) y `Read`, sin MCP
//!   (`--strict-mcp-config`) y solo con los ajustes del proyecto y locales.
//! - Sin las variables de clave de API en el entorno: usa la suscripción.
//! - `lumeart` delante en el `PATH`, `LUMEART_SESSION` apuntando a la sesión
//!   de la ventana y `LUMEART_RUN` para que sus cambios queden como del
//!   agente.
//! - El encargo va por la entrada estándar y las instrucciones en un archivo
//!   (`--append-system-prompt-file`): sin argumentos de varias líneas.
//! - `--output-format stream-json`: cada línea se traduce a `AgentEvent`.
//! - Proceso en su grupo (Unix) o Job Object (Windows): detener mata `claude`
//!   y todo lo que haya lanzado.
//!
//! Cada ejecución trabaja en `<work_dir>/run-<n>/`, con las imágenes de
//! referencia, las revisiones que exporte y una copia del stream.

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{Arc, Mutex};

use base64::Engine;
use lume_app::AgentRunner;
use lume_app::ports::{
    AgentEvent, AgentTask, BoxFuture, CANVAS_ATTACHMENT, CancellationToken, ContentPart,
    Conversation, EventSink, PortError, RunEnd, RunSummary, ToolBox, ToolOutput, Usage,
};
use lume_domain::discovery::{RUN_ENV, SESSION_ENV};
use process_wrap::tokio::{CommandWrap, KillOnDrop};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::critic::AgentRole;
use crate::prompts;

/// Variables de entorno que no llegan a `claude` (para que use la
/// suscripción y no una clave).
pub const REMOVED_ENV: [&str; 2] = ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"];

/// Mensaje cuando no está `claude`.
pub fn missing_claude() -> PortError {
    PortError::Unavailable(
        "no encuentro `claude` (Claude Code). Instálalo desde https://claude.com/claude-code \
         (por ejemplo, con `npm install -g @anthropic-ai/claude-code`) o elige otro proveedor \
         en Ajustes › Proveedores de IA"
            .into(),
    )
}

/// El plugin oficial `ralph-loop` de Claude Code (la variante A de
/// `lumeart eval`): se carga solo para la ejecución con `--plugin-dir` y su
/// Stop hook devuelve el encargo a Claude hasta que escribe la promesa o se
/// acaban las vueltas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RalphPlugin {
    /// Carpeta del plugin (la que tiene `.claude-plugin/plugin.json`).
    pub dir: PathBuf,
    /// Vueltas como mucho (`--max-iterations`).
    pub max_iterations: u32,
}

/// Con el plugin, el encargo va en este archivo de la carpeta de la
/// ejecución (el plugin lo pasa por la línea de órdenes de bash, que no
/// admite cualquier texto).
pub const RALPH_TASK_FILE: &str = "encargo.md";
/// Estado del plugin en la carpeta de la ejecución (`iteration: N`).
pub const RALPH_STATE_FILE: &str = ".claude/ralph-loop.local.md";

impl RalphPlugin {
    /// Lo que se le pide a Claude: el comando del plugin con la promesa de
    /// LumeArt y las vueltas.
    pub fn prompt(&self) -> String {
        format!(
            "/ralph-loop:ralph-loop Lee {RALPH_TASK_FILE} y haz lo que pide --completion-promise {} --max-iterations {}",
            lume_app::refine::PROMISE
                .trim_start_matches("<promise>")
                .trim_end_matches("</promise>"),
            self.max_iterations.max(1)
        )
    }
}

/// La vuelta en que va el plugin, según su archivo de estado.
pub fn ralph_iteration(state: &str) -> Option<u32> {
    state
        .lines()
        .find_map(|l| l.strip_prefix("iteration:"))
        .and_then(|n| n.trim().parse().ok())
}

/// Agente que lanza Claude Code.
#[derive(Debug, Clone)]
pub struct ClaudeCodeRunner {
    /// Ejecutable de `claude` (`None`: no está instalado).
    pub claude: Option<PathBuf>,
    /// Ejecutable de `lumeart`: su carpeta va delante en el `PATH`.
    pub lumeart: PathBuf,
    /// Archivo de estado de la sesión de la ventana (`LUMEART_SESSION`).
    pub session_file: PathBuf,
    /// Carpeta donde se crean las carpetas de cada ejecución.
    pub work_dir: PathBuf,
    /// Dibuja o revisa.
    pub role: AgentRole,
    /// Con el plugin oficial, que lleva el bucle (solo el artista).
    pub ralph: Option<RalphPlugin>,
    /// Sin restricciones (solo el artista): todas las herramientas de
    /// Claude Code y ningún permiso que pedir.
    pub full_access: bool,
}

impl ClaudeCodeRunner {
    pub fn new(
        claude: Option<PathBuf>,
        lumeart: PathBuf,
        session_file: PathBuf,
        work_dir: PathBuf,
    ) -> Self {
        Self {
            claude,
            lumeart,
            session_file,
            work_dir,
            role: AgentRole::Artist,
            ralph: None,
            full_access: false,
        }
    }

    /// Con acceso completo (o no): el artista puede usar cualquier
    /// herramienta, comando o archivo sin pedir permiso. El crítico sigue
    /// solo con `Read`.
    pub fn with_full_access(mut self, on: bool) -> Self {
        self.full_access = on;
        self
    }

    /// Si esta ejecución va sin restricciones.
    fn unrestricted(&self) -> bool {
        self.full_access && self.role == AgentRole::Artist
    }

    /// Con el plugin oficial `ralph-loop` llevando el bucle.
    pub fn with_ralph(mut self, plugin: RalphPlugin) -> Self {
        self.ralph = Some(plugin);
        self
    }

    /// El plugin, si dibuja con él.
    fn ralph(&self) -> Option<&RalphPlugin> {
        self.ralph
            .as_ref()
            .filter(|_| self.role == AgentRole::Artist)
    }

    /// Como crítico: solo `Read` (no puede ejecutar nada ni cambiar el
    /// dibujo), sus instrucciones, y el dibujo en un PNG de su carpeta.
    pub fn as_critic(mut self) -> Self {
        self.role = AgentRole::Critic;
        self
    }

    /// Carpeta de trabajo de una ejecución.
    pub fn run_dir(&self, task: &AgentTask) -> PathBuf {
        match self.role {
            AgentRole::Artist => self.work_dir.join(format!("run-{}", task.run)),
            AgentRole::Critic => self.work_dir.join(format!("run-{}-critica", task.run)),
        }
    }

    /// Argumentos de `claude` (como los de pxcli). El artista puede usar
    /// `lumeart`, leer imágenes y escribir sus notas (`./notas.md`); el
    /// crítico solo leer.
    pub fn args(&self, task: &AgentTask) -> Vec<String> {
        let animation =
            self.role == AgentRole::Artist && (task.anim.is_some() || task.vector.is_some());
        let (tools, mut allowed): (&str, Vec<&str>) = match self.role {
            // Con animación, también la herramienta `Edit`: cambiar unas
            // líneas de anim.js sin reescribirlo entero.
            AgentRole::Artist if animation => (
                "Bash,Read,Write,Edit",
                vec!["Bash(lumeart:*)", "Read", "Edit(./notas.md)"],
            ),
            AgentRole::Artist => (
                "Bash,Read,Write",
                // En Claude Code, escribir archivos se permite con reglas `Edit`.
                vec!["Bash(lumeart:*)", "Read", "Edit(./notas.md)"],
            ),
            AgentRole::Critic => ("Read", vec!["Read"]),
        };
        // Con animación, también su anim.js; con dibujo vectorial, su dibujo.js.
        if self.role == AgentRole::Artist && task.anim.is_some() {
            allowed.push("Edit(./anim.js)");
        }
        if self.role == AgentRole::Artist && task.vector.is_some() {
            allowed.push("Edit(./dibujo.js)");
        }
        let limits: Vec<&str> = if self.unrestricted() {
            // Acceso completo: todas sus herramientas y sin pedir permiso.
            vec!["--permission-mode", "bypassPermissions"]
        } else {
            ["--tools", tools, "--allowedTools"]
                .into_iter()
                .chain(allowed.iter().copied())
                .collect()
        };
        let mut args: Vec<String> = [
            "-p",
            "--output-format",
            "stream-json",
            "--verbose",
            "--include-partial-messages",
        ]
        .into_iter()
        .chain(limits)
        .chain([
            "--strict-mcp-config",
            "--setting-sources",
            "project,local",
            "--append-system-prompt-file",
            SYSTEM_FILE,
        ])
        .map(str::to_string)
        .collect();
        if !task.model.trim().is_empty() {
            args.push("--model".into());
            args.push(task.model.trim().to_string());
        }
        if let Some(Conversation::ClaudeSession(id)) = &task.resume {
            args.push("--resume".into());
            args.push(id.clone());
        }
        if let Some(plugin) = self.ralph() {
            args.push("--plugin-dir".into());
            args.push(plugin.dir.to_string_lossy().into_owned());
        }
        args
    }

    /// Variables de entorno que se fijan (las de [`REMOVED_ENV`] se quitan).
    pub fn env(&self, task: &AgentTask) -> Vec<(String, OsString)> {
        let mut dirs: Vec<PathBuf> = self
            .lumeart
            .parent()
            .map(Path::to_path_buf)
            .into_iter()
            .collect();
        if let Some(path) = std::env::var_os("PATH") {
            dirs.extend(std::env::split_paths(&path));
        }
        let path = std::env::join_paths(dirs).unwrap_or_default();
        vec![
            ("PATH".into(), path),
            (
                SESSION_ENV.into(),
                self.session_file.clone().into_os_string(),
            ),
            (RUN_ENV.into(), task.run.to_string().into()),
        ]
    }

    /// El comando completo: argumentos, carpeta, entrada y salidas, y el
    /// entorno sin claves.
    pub fn command(&self, claude: &Path, task: &AgentTask, dir: &Path) -> tokio::process::Command {
        let mut command = tokio::process::Command::new(claude);
        command
            .args(self.args(task))
            .current_dir(dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for name in REMOVED_ENV {
            command.env_remove(name);
        }
        for (name, value) in self.env(task) {
            command.env(name, value);
        }
        command
    }

    /// Prepara la carpeta de la ejecución: instrucciones, el dibujo (para el
    /// crítico) e imágenes de referencia. Devuelve el encargo para la entrada
    /// estándar.
    fn prepare(
        &self,
        task: &AgentTask,
        dir: &Path,
        drawing: Option<&[u8]>,
    ) -> Result<String, PortError> {
        let io = |e: std::io::Error| PortError::Io(format!("{}: {e}", dir.display()));
        std::fs::create_dir_all(dir).map_err(io)?;
        let mut system = match self.role {
            AgentRole::Artist => prompts::claude_code_system(task),
            AgentRole::Critic => prompts::critic_system(),
        };
        if self.unrestricted() {
            system.push_str(prompts::FULL_ACCESS_NOTE);
        }
        std::fs::write(dir.join(SYSTEM_FILE), system).map_err(io)?;
        if let Some(png) = drawing {
            std::fs::write(dir.join(DRAWING_FILE), png).map_err(io)?;
        }
        let mut references = Vec::new();
        let mut canvas = None;
        for attachment in &task.attachments {
            if attachment.label == CANVAS_ATTACHMENT {
                std::fs::write(dir.join(CANVAS_FILE), &attachment.data).map_err(io)?;
                canvas = Some(CANVAS_FILE);
                continue;
            }
            let ext = match attachment.media_type.as_str() {
                "image/jpeg" => "jpg",
                "image/gif" => "gif",
                "image/webp" => "webp",
                _ => "png",
            };
            let name = format!("referencia-{}.{ext}", references.len() + 1);
            std::fs::write(dir.join(&name), &attachment.data).map_err(io)?;
            references.push(name);
        }
        Ok(match self.role {
            AgentRole::Artist => prompts::claude_code_task(task, &references, canvas),
            AgentRole::Critic => prompts::critic_task_claude_code(task, DRAWING_FILE, &references),
        })
    }
}

/// El dibujo que mira el crítico, en su carpeta.
pub const DRAWING_FILE: &str = "dibujo.png";
/// El lienzo al empezar una vuelta del bucle, en la carpeta del artista.
pub const CANVAS_FILE: &str = "lienzo.png";
/// Las notas que escribe el artista (pasan al documento al terminar).
pub const NOTES_FILE: &str = "notas.md";
/// El código de la animación, si el encargo es de animación (pasa al
/// documento al terminar si cambió).
pub const ANIM_FILE: &str = "anim.js";
/// El código del dibujo vectorial, si el encargo es vectorial (pasa al
/// documento cada vez que cambia).
pub const VEC_FILE: &str = "dibujo.js";
/// La base de la animación, junto a `anim.js`.
pub const ANIM_BASE_FILE: &str = "base.png";

/// El PNG del lienzo que da `view_canvas`.
async fn canvas_png(tools: &ToolBox) -> Result<Vec<u8>, PortError> {
    let output = tools.call("view_canvas", "{}").await;
    output
        .content
        .into_iter()
        .find_map(|part| match part {
            ContentPart::Image { data, .. } => Some(data),
            _ => None,
        })
        .ok_or_else(|| PortError::Unavailable("no se pudo ver el dibujo para revisarlo".into()))
}

/// Archivo con las instrucciones de sistema, en la carpeta de la ejecución.
pub const SYSTEM_FILE: &str = "instrucciones.md";

impl AgentRunner for ClaudeCodeRunner {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        events: EventSink,
        cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        Box::pin(async move {
            let Some(claude) = self.claude.clone() else {
                return Err(missing_claude());
            };
            let dir = self.run_dir(&task);
            let drawing = match self.role {
                AgentRole::Critic => Some(canvas_png(&tools).await?),
                AgentRole::Artist => None,
            };
            let mut prompt = self.prepare(&task, &dir, drawing.as_deref())?;
            // Con el plugin: el encargo, en un archivo; a Claude, el comando.
            let ralph = self.ralph().cloned();
            if let Some(plugin) = &ralph {
                let io = |e: std::io::Error| PortError::Io(e.to_string());
                std::fs::write(dir.join(RALPH_TASK_FILE), &prompt).map_err(io)?;
                let _ = std::fs::remove_file(dir.join(RALPH_STATE_FILE));
                prompt = plugin.prompt();
            }
            let mut loops = 0u32;
            // El plugin da varias vueltas: los límites son por vuelta.
            let mut limits = task.limits;
            if let Some(plugin) = &ralph {
                let n = plugin.max_iterations.max(1);
                limits.max_steps = limits.max_steps.saturating_mul(n);
                limits.max_tokens = limits.max_tokens.saturating_mul(u64::from(n));
            }
            // Una animación: su anim.js y su base, como archivos.
            if let Some(anim) = &task.anim {
                let io = |e: std::io::Error| PortError::Io(e.to_string());
                std::fs::write(dir.join(ANIM_FILE), &anim.source).map_err(io)?;
                std::fs::write(dir.join(ANIM_BASE_FILE), &anim.base_png).map_err(io)?;
            }
            // Un dibujo vectorial: su dibujo.js, como archivo.
            if let Some(vector) = &task.vector {
                let io = |e: std::io::Error| PortError::Io(e.to_string());
                std::fs::write(dir.join(VEC_FILE), &vector.source).map_err(io)?;
            }
            // Lo último que se pasó al documento.
            let mut synced = task.vector.as_ref().map(|v| v.source.clone());

            let mut wrapped = CommandWrap::from(self.command(&claude, &task, &dir));
            #[cfg(unix)]
            wrapped.wrap(process_wrap::tokio::ProcessGroup::leader());
            #[cfg(windows)]
            wrapped.wrap(process_wrap::tokio::JobObject);
            wrapped.wrap(KillOnDrop);
            let mut child = wrapped.spawn().map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    missing_claude()
                } else {
                    PortError::Unavailable(format!("no se pudo lanzar claude: {e}"))
                }
            })?;

            // El encargo, por la entrada estándar.
            if let Some(mut stdin) = child.stdin().take() {
                let _ = stdin.write_all(prompt.as_bytes()).await;
                let _ = stdin.shutdown().await;
            }
            // Lo último que diga por la salida de error, por si falla.
            let stderr_tail = Arc::new(Mutex::new(Vec::<String>::new()));
            let mut stderr_task = None;
            if let Some(stderr) = child.stderr().take() {
                let tail = stderr_tail.clone();
                stderr_task = Some(tokio::spawn(async move {
                    let mut lines = BufReader::new(stderr).lines();
                    while let Ok(Some(line)) = lines.next_line().await {
                        let mut tail = tail.lock().unwrap_or_else(|e| e.into_inner());
                        if !line.trim().is_empty() {
                            tail.push(line);
                        }
                        let excess = tail.len().saturating_sub(20);
                        tail.drain(..excess);
                    }
                }));
            }
            let Some(stdout) = child.stdout().take() else {
                return Err(PortError::Unavailable("claude no tiene salida".into()));
            };
            let mut log = std::fs::File::create(dir.join("stream.jsonl")).ok();
            let mut lines = BufReader::new(stdout).lines();
            let mut parser = StreamParser::default();

            let stopped = loop {
                tokio::select! {
                    biased;
                    _ = cancel.cancelled() => break Some(RunEnd::Cancelled),
                    line = lines.next_line() => {
                        let Ok(Some(line)) = line else { break None };
                        if let Some(log) = log.as_mut() {
                            use std::io::Write;
                            let _ = writeln!(log, "{line}");
                        }
                        for event in parser.line(&line) {
                            events.emit(event);
                        }
                        // Cada cambio de dibujo.js pasa al documento en cuanto
                        // ocurre (el usuario lo ve crecer).
                        sync_vector(&dir, &mut synced, &tools).await;
                        // La vuelta del plugin (su archivo se borra al acabar).
                        if ralph.is_some()
                            && let Ok(state) = std::fs::read_to_string(dir.join(RALPH_STATE_FILE))
                            && let Some(n) = ralph_iteration(&state)
                        {
                            loops = loops.max(n);
                        }
                        if parser.result.is_none() {
                            if parser.steps() > limits.max_steps {
                                break Some(RunEnd::StepLimit);
                            }
                            if parser.usage().total() >= limits.max_tokens {
                                break Some(RunEnd::TokenLimit);
                            }
                        }
                    }
                }
            };
            let loops = ralph.as_ref().map(|_| loops.max(1));
            if let Some(end) = stopped {
                let _ = Box::into_pin(child.kill()).await;
                let mut summary = parser.summary(end);
                summary.steps = summary.steps.min(limits.max_steps);
                summary.loops = loops;
                return Ok(summary);
            }

            let status = child.wait().await;
            // Lo que dijo por la salida de error, entero (si un proceso hijo
            // la deja abierta, no se espera más de 2 s).
            if let Some(task) = stderr_task {
                let _ = tokio::time::timeout(std::time::Duration::from_secs(2), task).await;
            }
            // Sus notas pasan al documento.
            if self.role == AgentRole::Artist
                && let Ok(notes) = std::fs::read_to_string(dir.join(NOTES_FILE))
                && !notes.trim().is_empty()
            {
                let input = serde_json::json!({ "text": notes }).to_string();
                let _ = tools.call("write_notes", &input).await;
            }
            // Y su anim.js, si lo cambió.
            if self.role == AgentRole::Artist
                && let Some(anim) = &task.anim
                && let Ok(source) = std::fs::read_to_string(dir.join(ANIM_FILE))
                && source != anim.source
            {
                let input = serde_json::json!({ "source": source }).to_string();
                let _ = tools.call("write_anim", &input).await;
            }
            // Y su dibujo.js, si cambió desde la última vez.
            if self.role == AgentRole::Artist {
                sync_vector(&dir, &mut synced, &tools).await;
            }
            let finish = |end: RunEnd| {
                let mut summary = parser.summary(end);
                summary.loops = loops;
                summary
            };
            match &parser.result {
                Some(result) if result.subtype == "error_max_turns" => {
                    Ok(finish(RunEnd::StepLimit))
                }
                Some(result) if result.is_error || result.subtype != "success" => {
                    let detail = if result.text.trim().is_empty() {
                        result.subtype.clone()
                    } else {
                        result.text.trim().to_string()
                    };
                    Err(PortError::Provider(format!("Claude Code falló: {detail}")))
                }
                Some(_) => Ok(finish(RunEnd::Finished)),
                None => {
                    let tail = stderr_tail
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .join("\n");
                    let code = status
                        .map(|s| s.to_string())
                        .unwrap_or_else(|e| e.to_string());
                    Err(PortError::Provider(format!(
                        "claude terminó sin resultado ({code}){}{tail}",
                        if tail.is_empty() { "" } else { ": " }
                    )))
                }
            }
        })
    }
}

/// El `result` final del stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamResult {
    pub subtype: String,
    pub is_error: bool,
    pub turns: u32,
    pub text: String,
    pub usage: Usage,
}

/// Traduce las líneas de `--output-format stream-json` a `AgentEvent`.
/// Usa los mensajes completos (`assistant`, `user`, `result`); los trozos
/// parciales (`stream_event`) se ignoran.
#[derive(Debug, Default)]
pub struct StreamParser {
    /// Nombre de cada herramienta por su id (para los resultados).
    tool_names: HashMap<String, String>,
    /// Uso de cada mensaje del modelo (el último visto de cada uno).
    usage_by_message: Vec<(String, Usage)>,
    last_text: String,
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub result: Option<StreamResult>,
}

impl StreamParser {
    /// Eventos de una línea (ninguno si no se entiende).
    pub fn line(&mut self, line: &str) -> Vec<AgentEvent> {
        let Ok(msg) = serde_json::from_str::<Value>(line) else {
            return Vec::new();
        };
        let mut events = Vec::new();
        match msg["type"].as_str() {
            Some("system") if msg["subtype"] == "init" => {
                self.session_id = msg["session_id"].as_str().map(str::to_string);
                self.model = msg["model"].as_str().map(str::to_string);
            }
            Some("assistant") => {
                let message = &msg["message"];
                if let Some(id) = message["id"].as_str() {
                    let usage = usage_of(&message["usage"]);
                    match self.usage_by_message.iter_mut().find(|(m, _)| m == id) {
                        Some((_, u)) => *u = usage,
                        None => self.usage_by_message.push((id.to_string(), usage)),
                    }
                }
                for block in message["content"].as_array().into_iter().flatten() {
                    match block["type"].as_str() {
                        Some("text") => {
                            let text = block["text"].as_str().unwrap_or_default();
                            if !text.trim().is_empty() {
                                self.last_text = text.to_string();
                                events.push(AgentEvent::Text(text.to_string()));
                            }
                        }
                        Some("thinking") => {
                            let text = block["thinking"].as_str().unwrap_or_default();
                            if !text.trim().is_empty() {
                                events.push(AgentEvent::Thinking(text.to_string()));
                            }
                        }
                        Some("tool_use") => {
                            let id = block["id"].as_str().unwrap_or_default().to_string();
                            let name = block["name"].as_str().unwrap_or_default().to_string();
                            self.tool_names.insert(id, name.clone());
                            events.push(AgentEvent::ToolCall {
                                name,
                                input_json: block["input"].to_string(),
                            });
                        }
                        _ => {}
                    }
                }
            }
            Some("user") => {
                for block in msg["message"]["content"].as_array().into_iter().flatten() {
                    if block["type"] != "tool_result" {
                        continue;
                    }
                    let id = block["tool_use_id"].as_str().unwrap_or_default();
                    let name = self.tool_names.get(id).cloned().unwrap_or_default();
                    events.push(AgentEvent::ToolResult {
                        name,
                        output: ToolOutput {
                            content: result_content(&block["content"]),
                            is_error: block["is_error"].as_bool().unwrap_or(false),
                        },
                    });
                }
            }
            Some("result") => {
                let result = StreamResult {
                    subtype: msg["subtype"].as_str().unwrap_or_default().to_string(),
                    is_error: msg["is_error"].as_bool().unwrap_or(false),
                    turns: msg["num_turns"].as_u64().unwrap_or(0) as u32,
                    text: msg["result"].as_str().unwrap_or_default().to_string(),
                    usage: usage_of(&msg["usage"]),
                };
                events.push(AgentEvent::Usage(result.usage));
                self.result = Some(result);
            }
            _ => {}
        }
        events
    }

    /// Pasos: los turnos del resultado o, mientras tanto, los mensajes del
    /// modelo vistos.
    pub fn steps(&self) -> u32 {
        match &self.result {
            Some(r) if r.turns > 0 => r.turns,
            _ => self.usage_by_message.len() as u32,
        }
    }

    /// Uso: el del resultado o, mientras tanto, la suma de los mensajes.
    pub fn usage(&self) -> Usage {
        if let Some(r) = &self.result {
            return r.usage;
        }
        let mut total = Usage::default();
        for (_, u) in &self.usage_by_message {
            total.add(*u);
        }
        total
    }

    pub fn summary(&self, end: RunEnd) -> RunSummary {
        let final_text = match &self.result {
            Some(r) if !r.text.trim().is_empty() => r.text.clone(),
            _ => self.last_text.clone(),
        };
        // La sesión de Claude Code se puede retomar si terminó bien.
        let conversation = match (&self.session_id, end) {
            (Some(id), RunEnd::Finished) => Some(Conversation::ClaudeSession(id.clone())),
            _ => None,
        };
        RunSummary {
            steps: self.steps(),
            usage: self.usage(),
            final_text,
            end,
            conversation,
            loops: None,
        }
    }
}

/// Uso con la forma de la API de Anthropic.
fn usage_of(v: &Value) -> Usage {
    let n = |k: &str| v[k].as_u64().unwrap_or(0);
    Usage {
        input_tokens: n("input_tokens") + n("cache_creation_input_tokens"),
        output_tokens: n("output_tokens"),
        cached_tokens: n("cache_read_input_tokens"),
    }
}

/// Contenido de un `tool_result`: texto o lista de texto e imágenes.
fn result_content(v: &Value) -> Vec<ContentPart> {
    if let Some(text) = v.as_str() {
        return vec![ContentPart::Text(text.to_string())];
    }
    v.as_array()
        .into_iter()
        .flatten()
        .filter_map(|part| match part["type"].as_str() {
            Some("text") => Some(ContentPart::Text(
                part["text"].as_str().unwrap_or_default().to_string(),
            )),
            Some("image") => {
                let source = &part["source"];
                let data = base64::engine::general_purpose::STANDARD
                    .decode(source["data"].as_str().unwrap_or_default())
                    .ok()?;
                Some(ContentPart::Image {
                    media_type: source["media_type"]
                        .as_str()
                        .unwrap_or("image/png")
                        .to_string(),
                    data,
                })
            }
            _ => None,
        })
        .collect()
}

/// Pasa `dibujo.js` al documento si cambió desde `synced`.
async fn sync_vector(dir: &Path, synced: &mut Option<String>, tools: &ToolBox) {
    let Some(last) = synced.as_mut() else {
        return;
    };
    if let Ok(source) = std::fs::read_to_string(dir.join(VEC_FILE))
        && source != *last
    {
        let input = serde_json::json!({ "source": source }).to_string();
        let _ = tools.call("write_vec", &input).await;
        *last = source;
    }
}
