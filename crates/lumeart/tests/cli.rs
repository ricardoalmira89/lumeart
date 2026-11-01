//! La CLI contra una API en proceso (sesión + servidor en 127.0.0.1): los
//! tests de la CLI de pxcli portados, `export` relativo, `--session` y
//! `LUMEART_SESSION`, y el error sin sesión con el binario real.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::Parser;
use lume_api::{ApiServer, write_state_file};
use lume_app::testing::MemoryStore;
use lume_app::{DocumentStore, Session};
use lume_cli::{Cli, Context};
use lume_domain::Document;
use lume_store::FsStore;

/// Una sesión servida por la API, con su archivo de estado.
struct Running {
    session: Session,
    server: ApiServer,
    state: PathBuf,
}

async fn serve(store: Arc<dyn DocumentStore>, w: i64, h: i64, state: PathBuf) -> Running {
    let session = Session::spawn(Document::new("cli", w, h).unwrap(), store.clone());
    let server = ApiServer::start(session.handle.clone(), store)
        .await
        .unwrap();
    write_state_file(&state, &server.info()).unwrap();
    Running {
        session,
        server,
        state,
    }
}

impl Running {
    async fn stop(self) {
        self.server.shutdown().await.unwrap();
        self.session.close().await;
    }
}

/// Resultado de ejecutar `lumeart <args>` en proceso.
#[derive(Debug)]
struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

/// Ejecuta la CLI en un hilo aparte (es bloqueante) con este entorno.
async fn lumeart(
    args: &[&str],
    cwd: &Path,
    env: Option<&Path>,
    default: Option<&Path>,
    stdin: &str,
) -> Out {
    let mut full = vec!["lumeart".to_string()];
    full.extend(args.iter().map(|s| s.to_string()));
    let cwd = cwd.to_path_buf();
    let env = env.map(|p| p.to_string_lossy().into_owned());
    let default = default.map(Path::to_path_buf);
    let stdin = stdin.to_string();
    tokio::task::spawn_blocking(move || {
        let cli = Cli::try_parse_from(full).unwrap();
        let (mut input, mut stdout, mut stderr) = (stdin.as_bytes(), Vec::new(), Vec::new());
        let mut ctx = Context {
            cwd,
            session_env: env,
            agent_run: None,
            default_session: default,
            stdin: &mut input,
            stdout: &mut stdout,
            stderr: &mut stderr,
        };
        let code = lume_cli::execute(
            cli.command.as_ref().unwrap(),
            cli.session.as_deref(),
            &mut ctx,
        );
        Out {
            code,
            stdout: String::from_utf8(stdout).unwrap(),
            stderr: String::from_utf8(stderr).unwrap(),
        }
    })
    .await
    .unwrap()
}

// Portado de TestGetPixelCmd_PrintsColor.
#[tokio::test(flavor = "multi_thread")]
async fn get_pixel_prints_the_color() {
    let dir = tempfile::tempdir().unwrap();
    let run = serve(Arc::new(FsStore), 4, 4, dir.path().join("session.json")).await;
    let s = Some(run.state.as_path());
    let out = lumeart(&["set_pixel", "0", "0", "#00ff00"], dir.path(), None, s, "").await;
    assert_eq!(
        (out.code, out.stdout.as_str(), out.stderr.as_str()),
        (0, "ok\n", "")
    );
    let out = lumeart(&["get_pixel", "0", "0"], dir.path(), None, s, "").await;
    assert_eq!((out.code, out.stdout.as_str()), (0, "ok #00ff00ff\n"));
    run.stop().await;
}

// Portado de TestUndoCmd_RevertsCanvasState.
#[tokio::test(flavor = "multi_thread")]
async fn undo_reverts_the_canvas() {
    let dir = tempfile::tempdir().unwrap();
    let run = serve(Arc::new(FsStore), 2, 2, dir.path().join("session.json")).await;
    let s = Some(run.state.as_path());
    lumeart(&["set_pixel", "0", "0", "#ff0000"], dir.path(), None, s, "").await;
    let out = lumeart(&["undo"], dir.path(), None, s, "").await;
    assert_eq!((out.code, out.stdout.as_str()), (0, "ok\n"));
    let out = lumeart(&["get_pixel", "0", "0"], dir.path(), None, s, "").await;
    assert_eq!(out.stdout, "ok #00000000\n");
    let out = lumeart(&["redo"], dir.path(), None, s, "").await;
    assert_eq!(out.stdout, "ok\n");
    let out = lumeart(&["redo"], dir.path(), None, s, "").await;
    assert_eq!(
        (out.code, out.stdout.as_str(), out.stderr.as_str()),
        (1, "", "Error: err no_history nothing to redo\n")
    );
    run.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn errors_go_to_stderr_with_exit_code_1() {
    let dir = tempfile::tempdir().unwrap();
    let run = serve(Arc::new(FsStore), 4, 4, dir.path().join("session.json")).await;
    let s = Some(run.state.as_path());
    let out = lumeart(
        &["set_pixel", "-1", "10", "#ff0000"],
        dir.path(),
        None,
        s,
        "",
    )
    .await;
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stderr,
        "Error: err out_of_bounds pixel (-1,10) outside canvas\n"
    );
    let out = lumeart(
        &["fill_rect", "0", "0", "0", "2", "red"],
        dir.path(),
        None,
        s,
        "",
    )
    .await;
    assert_eq!(out.stderr, "Error: err invalid_args w must be > 0\n");
    let out = lumeart(&["set_pixel", "0", "0", "nocolor"], dir.path(), None, s, "").await;
    assert_eq!(
        out.stderr,
        "Error: err invalid_color expected hex color or named color\n"
    );
    let out = lumeart(&["new", "16x8"], dir.path(), None, s, "").await;
    assert_eq!(out.stdout, "ok 16x8\n");
    run.stop().await;
}

// Portado de TestExportCmd_ResolvesAbsolutePath y PropagatesIOError.
#[tokio::test(flavor = "multi_thread")]
async fn export_resolves_relative_paths_where_the_command_runs() {
    let dir = tempfile::tempdir().unwrap();
    let game = dir.path().join("mi juego");
    std::fs::create_dir(&game).unwrap();
    let run = serve(Arc::new(FsStore), 3, 2, dir.path().join("session.json")).await;
    let s = Some(run.state.as_path());
    lumeart(&["set_pixel", "2", "1", "red"], &game, None, s, "").await;
    let out = lumeart(&["export", "--scale", "4", "cofre.png"], &game, None, s, "").await;
    assert_eq!((out.code, out.stdout.as_str()), (0, "ok\n"));
    let png = lume_store::png_io::read_png(&game.join("cofre.png")).unwrap();
    assert_eq!((png.width(), png.height()), (12, 8));
    assert_eq!(png.get(11, 7).unwrap().to_string(), "#ff0000ff");

    // En un batch también, relativo a donde se ejecuta.
    let out = lumeart(&["batch", "export sub/../b.png"], &game, None, s, "").await;
    assert_eq!(out.stdout, "batch: 1 ok, 0 failed\n");
    assert!(game.join("b.png").is_file());

    let out = lumeart(&["export", "no/existe/x.png"], &game, None, s, "").await;
    assert_eq!(out.code, 1);
    assert_eq!(
        out.stderr,
        format!(
            "Error: err io open {}: no such file or directory\n",
            game.join("no/existe/x.png").display()
        )
    );
    run.stop().await;
}

/// El guion `batch-1` de la T:8, enviado por la CLI: misma salida que pxcli.
#[tokio::test(flavor = "multi_thread")]
async fn batch_prints_like_pxcli() {
    let dir = tempfile::tempdir().unwrap();
    let run = serve(
        Arc::new(MemoryStore::new()),
        4,
        4,
        dir.path().join("session.json"),
    )
    .await;
    let s = Some(run.state.as_path());
    let script = include_str!("../../lume-domain/tests/fixtures/batch-1.txt");
    let expected = include_str!("../../lume-domain/tests/fixtures/batch-1.pxcli.txt");
    let (stdout, rest) = expected.split_once("--- stderr ---\n").unwrap();
    let (stderr, _) = rest.split_once("--- exit ").unwrap();
    // Por la entrada estándar, desde la carpeta /BATCHDIR (como el script de captura).
    let out = lumeart(&["batch"], Path::new("/BATCHDIR"), None, s, script).await;
    assert_eq!(out.stdout, stdout);
    assert_eq!(out.stderr, stderr);
    assert_eq!(out.code, 1);

    let out = lumeart(
        &["batch", "set_pixel 0 0 red; get_pixel 0 0"],
        dir.path(),
        None,
        s,
        "",
    )
    .await;
    assert_eq!(
        out.stdout,
        "line 1: get_pixel 0 0 -> ok #ff0000ff\nbatch: 2 ok, 0 failed\n"
    );
    assert_eq!((out.code, out.stderr.as_str()), (0, ""));
    let out = lumeart(&["batch", "# solo un comentario"], dir.path(), None, s, "").await;
    assert_eq!(
        (out.code, out.stderr.as_str()),
        (1, "Error: err invalid_args batch is empty\n")
    );
    run.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn session_flag_and_env_choose_the_session() {
    let dir = tempfile::tempdir().unwrap();
    let a = serve(
        Arc::new(MemoryStore::new()),
        2,
        2,
        dir.path().join("a.json"),
    )
    .await;
    let b = serve(
        Arc::new(MemoryStore::new()),
        2,
        2,
        dir.path().join("b.json"),
    )
    .await;
    let default = dir.path().join("default.json");
    let a_path = a.state.to_string_lossy().into_owned();
    let b_path = b.state.to_string_lossy().into_owned();

    lumeart(
        &["--session", &a_path, "set_pixel", "0", "0", "red"],
        dir.path(),
        None,
        None,
        "",
    )
    .await;
    lumeart(
        &["set_pixel", "0", "0", "blue"],
        dir.path(),
        Some(&b.state),
        Some(&default),
        "",
    )
    .await;

    let out = lumeart(
        &["--session", &a_path, "get_pixel", "0", "0"],
        dir.path(),
        Some(&b.state),
        None,
        "",
    )
    .await;
    assert_eq!(
        out.stdout, "ok #ff0000ff\n",
        "--session gana a LUMEART_SESSION"
    );
    let out = lumeart(
        &["get_pixel", "0", "0"],
        dir.path(),
        Some(&b.state),
        Some(&default),
        "",
    )
    .await;
    assert_eq!(
        out.stdout, "ok #0000ffff\n",
        "LUMEART_SESSION gana al de por defecto"
    );
    let out = lumeart(
        &["--socket", &b_path, "get_pixel", "0", "0"],
        dir.path(),
        None,
        None,
        "",
    )
    .await;
    assert_eq!(
        out.stdout, "ok #0000ffff\n",
        "--socket (pxcli) es un alias de --session"
    );
    a.stop().await;
    b.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_dead_session_gives_a_clear_error() {
    let dir = tempfile::tempdir().unwrap();
    let run = serve(
        Arc::new(MemoryStore::new()),
        2,
        2,
        dir.path().join("session.json"),
    )
    .await;
    let state = run.state.clone();
    run.stop().await;
    let out = lumeart(&["get_pixel", "0", "0"], dir.path(), None, Some(&state), "").await;
    assert_eq!(out.code, 1);
    assert!(
        out.stderr
            .starts_with("Error: err daemon_not_running the session in "),
        "{}",
        out.stderr
    );
    assert!(out.stderr.contains("lumeart start"));
}

/// Con el binario de verdad: sin sesión abierta, código distinto de cero y
/// un mensaje claro.
#[test]
fn binary_without_a_session_fails_clearly() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("no-hay-sesion.json");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args([
            "--session",
            missing.to_str().unwrap(),
            "set_pixel",
            "0",
            "0",
            "red",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr,
        format!(
            "Error: err daemon_not_running no LumeArt session is running ({} not found); start one with \"lumeart start\"\n",
            missing.display()
        )
    );
    // `--version` imprime la versión (sin comando se abre el studio).
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .arg("--version")
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("lumeart {}\n", env!("CARGO_PKG_VERSION"))
    );
}
