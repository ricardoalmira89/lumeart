//! `lumeart start`, `stop` y `serve` (el proceso en segundo plano).
//!
//! `start` relanza el propio ejecutable con el subcomando oculto `serve`,
//! desligado de la terminal, y espera a que la API responda. Una sesión está
//! viva si su API contesta con el token de su archivo de estado: así se
//! detectan los estados huérfanos en Linux, Windows y macOS sin señales.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lume_api::{ApiServer, write_state_file};
use lume_app::Session;
use lume_cli::{StartArgs, http, parse_canvas_size, read_session};
use lume_domain::{Document, MAX_CANVAS_SIZE, SessionInfo};
use lume_store::FsStore;

/// Tiempo máximo para que una sesión nueva responda.
const START_TIMEOUT: Duration = Duration::from_secs(10);
/// Tiempo máximo para que una sesión se cierre.
const STOP_TIMEOUT: Duration = Duration::from_secs(5);
const POLL: Duration = Duration::from_millis(30);

/// ¿Contesta la API de esta sesión con su token?
pub fn is_alive(info: &SessionInfo) -> bool {
    http::request_with_timeout(
        info.port,
        &info.token,
        "GET",
        "/v1/canvas.png",
        b"",
        Duration::from_millis(500),
    )
    .is_ok_and(|r| r.status == 200)
}

/// Archivo de registro del proceso en segundo plano (junto al de estado).
fn log_path(state: &Path) -> PathBuf {
    state.with_extension("log")
}

/// `lumeart start`: abre una sesión en segundo plano e imprime su PID.
pub fn start(
    args: &StartArgs,
    state: &Path,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    match try_start(args, state) {
        Ok(pid) => {
            let _ = writeln!(stdout, "{pid}");
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "Error: {message}");
            1
        }
    }
}

fn validate(args: &StartArgs) -> Result<(i64, i64), String> {
    let (w, h) = parse_canvas_size(&args.size).map_err(|e| e.0)?;
    let max = i64::from(MAX_CANVAS_SIZE);
    if w > max || h > max {
        return Err(format!(
            "err invalid_args width and height must be between 1 and {MAX_CANVAS_SIZE}"
        ));
    }
    if args.scale <= 0 {
        return Err(format!("invalid scale {}: must be > 0", args.scale));
    }
    if !args.headless && !has_display() {
        return Err(
            "err renderer_unavailable no hay pantalla para abrir la ventana; usa --headless"
                .to_string(),
        );
    }
    Ok((w, h))
}

/// ¿Hay pantalla para abrir una ventana? En Linux y BSD hace falta `DISPLAY`
/// (X11) o `WAYLAND_DISPLAY`; en Windows y macOS siempre la hay.
pub fn has_display() -> bool {
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        return true;
    }
    ["DISPLAY", "WAYLAND_DISPLAY"]
        .iter()
        .any(|v| std::env::var_os(v).is_some_and(|s| !s.is_empty()))
}

fn try_start(args: &StartArgs, state: &Path) -> Result<u32, String> {
    let (w, h) = validate(args)?;
    if state.exists() {
        match read_session(state) {
            Ok(info) if is_alive(&info) => {
                return Err(format!(
                    "err daemon_already_running ya hay una sesión abierta (pid {}, puerto {}); ciérrala con \"lumeart stop\"",
                    info.pid, info.port
                ));
            }
            // Estado huérfano (proceso muerto o archivo roto): se limpia.
            _ => std::fs::remove_file(state).map_err(|e| format!("err io {e}"))?,
        }
    }
    if let Some(dir) = state.parent() {
        std::fs::create_dir_all(dir).map_err(|e| format!("err io {e}"))?;
    }

    let exe = std::env::current_exe().map_err(|e| format!("err io {e}"))?;
    let log = std::fs::File::create(log_path(state)).map_err(|e| format!("err io {e}"))?;
    let mut command = Command::new(exe);
    command
        .arg("--session")
        .arg(state)
        .args([
            "serve",
            "--size",
            &format!("{w}x{h}"),
            if args.headless {
                "--headless=true"
            } else {
                "--headless=false"
            },
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log));
    detach(&mut command);
    let mut child = command.spawn().map_err(|e| format!("err io {e}"))?;
    wait_until_ready(&mut child, state)
}

/// Separa el proceso de la terminal para que siga vivo al cerrarla.
fn detach(command: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
        stop_inheriting_std_handles();
    }
}

/// En Windows, un proceso hijo hereda *todos* los handles heredables del
/// padre, no solo los tres que se le dan. Si `lumeart start` se lanzó con la
/// salida en una tubería (un script, `Command::output()` de un test), la
/// sesión se quedaría con esa tubería abierta mientras viva, y quien lee la
/// salida de `lumeart start` no terminaría nunca de leer. Por eso, antes de
/// lanzarla, la entrada y las salidas propias dejan de ser heredables; la
/// sesión recibe las suyas (`Stdio::null()`) y los demás hijos que se lancen
/// con `Stdio::inherit()` siguen funcionando, porque la biblioteca estándar
/// duplica el handle al dárselo.
///
/// Es el único `unsafe` del proyecto (lo comprueba `tests/arquitectura.rs`):
/// la biblioteca estándar no tiene una forma segura de cambiar la herencia de
/// un handle.
#[cfg(windows)]
#[allow(unsafe_code)]
fn stop_inheriting_std_handles() {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Foundation::{HANDLE_FLAG_INHERIT, SetHandleInformation};

    for handle in [
        std::io::stdin().as_raw_handle(),
        std::io::stdout().as_raw_handle(),
        std::io::stderr().as_raw_handle(),
    ] {
        if handle.is_null() {
            continue;
        }
        // SAFETY: `SetHandleInformation` solo cambia un indicador del handle y
        // no toca memoria de Rust. Los handles son los de la entrada y salidas
        // estándar del propio proceso, que siguen abiertos mientras vive; si
        // alguno no es válido, la llamada devuelve un error, que se ignora: lo
        // peor que pasa es lo de antes (que la sesión lo herede).
        unsafe {
            SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0);
        }
    }
}

fn wait_until_ready(child: &mut Child, state: &Path) -> Result<u32, String> {
    let deadline = Instant::now() + START_TIMEOUT;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            return Err(format!(
                "err daemon_start_failed la sesión terminó al arrancar ({status}); mira {}",
                log_path(state).display()
            ));
        }
        if let Ok(info) = read_session(state)
            && info.pid == child.id()
            && is_alive(&info)
        {
            return Ok(info.pid);
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            return Err("timed out waiting for daemon to start".to_string());
        }
        std::thread::sleep(POLL);
    }
}

/// `lumeart stop`: cierra la sesión y espera a que termine.
pub fn stop(state: &Path, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    match try_stop(state) {
        Ok(()) => {
            let _ = writeln!(stdout, "ok");
            0
        }
        Err(message) => {
            let _ = writeln!(stderr, "Error: {message}");
            1
        }
    }
}

fn try_stop(state: &Path) -> Result<(), String> {
    let info = read_session(state).map_err(|e| e.0)?;
    if lume_cli::call(&info, state, "/v1/stop", "").is_err() {
        // No contesta: era un estado huérfano.
        let _ = std::fs::remove_file(state);
        return Err(format!(
            "err daemon_not_running the session in {} was not running (stale state removed)",
            state.display()
        ));
    }
    let deadline = Instant::now() + STOP_TIMEOUT;
    while is_alive(&info) || state.exists() {
        if !is_alive(&info) && Instant::now() > deadline {
            // El proceso ya no contesta pero dejó el archivo: se borra.
            let _ = std::fs::remove_file(state);
            break;
        }
        if Instant::now() > deadline + STOP_TIMEOUT {
            return Err("timed out waiting for daemon to stop".to_string());
        }
        std::thread::sleep(POLL);
    }
    Ok(())
}

/// `lumeart serve` (interno): sirve una sesión hasta que le pidan parar.
pub fn serve(args: &StartArgs, state: &Path) -> i32 {
    let (w, h) = match parse_canvas_size(&args.size) {
        Ok(size) => size,
        Err(e) => {
            eprintln!("Error: {}", e.0);
            return 1;
        }
    };
    if !args.headless {
        return run_with_window(Some(state), true, w, h, lume_ui::Screen::Studio);
    }
    let runtime = tokio::runtime::Runtime::new().expect("runtime de tokio");
    let result = runtime.block_on(async {
        let document = Document::new("sin título", w, h).map_err(|e| e.to_string())?;
        let store: Arc<dyn lume_app::DocumentStore> = Arc::new(FsStore);
        let session = Session::spawn(document, store.clone());
        let art = crate::mcp::local_art(
            session.handle.clone(),
            store.clone(),
            settings_store(),
            work_dir(),
        );
        let server = ApiServer::start_with_art(session.handle.clone(), store, Some(Arc::new(art)))
            .await
            .map_err(|e| e.to_string())?;
        let info = server.info();
        write_state_file(state, &info).map_err(|e| e.to_string())?;
        eprintln!(
            "lumeart: sesión en 127.0.0.1:{} (pid {})",
            info.port, info.pid
        );

        let stop = server.stop_requested();
        tokio::select! {
            _ = stop.cancelled() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
        server.shutdown().await.map_err(|e| e.to_string())?;
        session.close().await;
        // Solo se borra si sigue siendo el nuestro.
        if read_session(state).is_ok_and(|i| i.pid == info.pid) {
            let _ = std::fs::remove_file(state);
        }
        remove_work_dir();
        eprintln!("lumeart: sesión cerrada");
        Ok::<_, String>(())
    });
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: {e}");
            1
        }
    }
}

/// `lumeart` sin argumentos: el studio (ventana + sesión + API). Publica su
/// archivo de estado salvo que ya haya otra sesión viva en él.
pub fn studio(state: Option<&Path>) -> i32 {
    run_with_window(state, false, 32, 32, lume_ui::Screen::Home)
}

/// Lista de recientes en la carpeta de datos del usuario (junto al archivo de
/// estado por defecto).
fn recents() -> Option<Arc<dyn lume_app::RecentStore>> {
    let dir = lume_api::default_state_path()?.parent()?.to_path_buf();
    Some(Arc::new(lume_store::FsRecents::new(
        dir.join("recents.json"),
    )))
}

/// Carpeta de autoguardado de las pestañas (se restauran al abrir).
fn autosave_dir() -> Option<PathBuf> {
    Some(
        lume_api::default_state_path()?
            .parent()?
            .join("autoguardado"),
    )
}

/// Carpeta temporal del agente de este proceso (Claude Code y encargos).
fn work_dir() -> PathBuf {
    std::env::temp_dir().join(format!("lumeart-agente-{}", std::process::id()))
}

/// Borra la carpeta temporal del agente al cerrar.
fn remove_work_dir() {
    let dir = work_dir();
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// `config.toml` en la carpeta de configuración del usuario
/// (`~/.config/lumeart/` en Linux).
pub fn config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "lumeart").map(|d| d.config_dir().join("config.toml"))
}

/// Ajustes guardados en `config.toml` (en memoria si no hay carpeta personal).
pub fn settings_store() -> Arc<dyn lume_app::SettingsStore> {
    match config_path() {
        Some(path) => Arc::new(lume_store::FsSettings::new(path)),
        None => Arc::new(lume_app::testing::MemorySettingsStore::default()),
    }
}

/// Sesión con API y ventana en el hilo principal. Cerrar la ventana cierra la
/// sesión; `lumeart stop` cierra la ventana.
fn run_with_window(
    state: Option<&Path>,
    always_publish: bool,
    w: i64,
    h: i64,
    screen: lume_ui::Screen,
) -> i32 {
    let runtime = tokio::runtime::Runtime::new().expect("runtime de tokio");
    let settings = settings_store();
    // El agente: Claude Code habla con esta sesión por su propio archivo de
    // estado (aunque la sesión publicada sea otra) y trabaja en una carpeta
    // temporal; los encargos por MCP, en subcarpetas de la misma.
    let work_dir = work_dir();
    // Peticiones de otros agentes (MCP) a esta ventana, por la API.
    let window = lume_app::studio::WindowChannel::default();
    let started = runtime.block_on(async {
        let document = Document::new("sin título", w, h).map_err(|e| e.to_string())?;
        let store: Arc<dyn lume_app::DocumentStore> = Arc::new(FsStore);
        let session = Session::spawn(document, store.clone());
        let art = Arc::new(crate::mcp::local_art(
            session.handle.clone(),
            store.clone(),
            settings.clone(),
            work_dir.clone(),
        ));
        let server = ApiServer::start_full(
            session.handle.clone(),
            store,
            Some(art.clone() as Arc<dyn lume_app::ArtService>),
            Some(window.clone()),
        )
        .await
        .map_err(|e| e.to_string())?;
        Ok::<_, String>((session, server, art))
    });
    let (session, server, art) = match started {
        Ok(started) => started,
        Err(e) => {
            eprintln!("Error: {e}");
            return 1;
        }
    };
    let info = server.info();
    let publish = state
        .filter(|path| always_publish || !read_session(path).is_ok_and(|other| is_alive(&other)));
    if let Some(path) = publish
        && let Err(e) = write_state_file(path, &info)
    {
        eprintln!("lumeart: no se pudo escribir {}: {e}", path.display());
    }

    let agent_session = work_dir.join("session.json");
    if let Err(e) = std::fs::create_dir_all(&work_dir)
        .map_err(|e| e.to_string())
        .and_then(|()| write_state_file(&agent_session, &info).map_err(|e| e.to_string()))
    {
        eprintln!("lumeart: no se pudo preparar la carpeta del agente: {e}");
    }
    let secrets: Arc<dyn lume_app::SecretStore> = Arc::new(lume_store::KeyringSecrets::default());
    let claude_path = lume_agent::find_claude();
    let (vec_tools, vec_services) = crate::vec_cmd::window_vector(work_dir.clone());
    let tab_vec_tools = vec_tools.clone();
    let agent = crate::agent::WindowAgent {
        session: session.handle.clone(),
        store: Arc::new(FsStore),
        secrets: secrets.clone(),
        claude: claude_path.clone(),
        lumeart: std::env::current_exe().unwrap_or_else(|_| PathBuf::from("lumeart")),
        session_file: agent_session.clone(),
        work_dir: work_dir.clone(),
        anim: Some(Arc::new(lume_script::QuickJsAnim)),
        vector: Some(vec_tools),
        ralph: None,
    };
    let result = lume_ui::run_studio(lume_ui::StudioOptions {
        session: session.handle.clone(),
        stop: server.stop_requested(),
        runtime: runtime.handle().clone(),
        store: Arc::new(FsStore),
        recents: recents(),
        dialogs: Arc::new(lume_ui::NativeDialogs),
        settings,
        secrets,
        claude_path,
        agent: Some(Arc::new(agent)),
        server: Some(format!("127.0.0.1:{}", info.port)),
        screen,
        art: Some(art),
        anim_engine: Some(Arc::new(lume_script::QuickJsAnim)),
        // También montan el agente de las pestañas con sesión propia: con
        // las mismas herramientas de dibujo.js que el de la ventana.
        variant_agents: Some(Arc::new(crate::mcp::JobAgents {
            vector: Some(tab_vec_tools),
            ..crate::mcp::job_agents(Arc::new(FsStore), work_dir.join("variantes"))
        })),
        vector: Some(vec_services),
        autosave: autosave_dir(),
        window: Some(window),
    });

    runtime.block_on(async {
        let _ = server.shutdown().await;
        session.close().await;
    });
    let _ = std::fs::remove_file(&agent_session);
    remove_work_dir();
    if let Some(path) = publish
        && read_session(path).is_ok_and(|i| i.pid == info.pid)
    {
        let _ = std::fs::remove_file(path);
    }
    match result {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("Error: err renderer_unavailable {e}");
            1
        }
    }
}
