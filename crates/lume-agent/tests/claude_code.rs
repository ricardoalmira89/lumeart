//! `ClaudeCodeRunner` con un `claude` falso: un guion que apunta cómo lo
//! lanzaron y emite el `stream.jsonl` grabado de una ejecución real de pxcli
//! (`reference/runs/…-una-seta-roja-con-puntos-bla`, en `fixtures/`).

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use lume_agent::ClaudeCodeRunner;
use lume_agent::claude_code::{REMOVED_ENV, SYSTEM_FILE, StreamParser, missing_claude};
use lume_app::AgentRunner;
use lume_app::ports::{
    AgentEvent, AgentLimits, AgentTask, Attachment, CancellationToken, ContentPart, EventSink,
    PortError, RunEnd,
};
use lume_app::testing::FakeTools;

const SETA: &str = include_str!("fixtures/claude_code/seta.stream.jsonl");

fn task(run: u64) -> AgentTask {
    AgentTask {
        run,
        goal: "una seta roja con puntos blancos".into(),
        context: String::new(),
        attachments: vec![],
        model: "sonnet".into(),
        limits: AgentLimits::default(),
        canvas: (32, 32),
        palette: None,
        continuing: false,
        resume: None,
        anim: None,
        vector: None,
    }
}

fn runner(claude: Option<PathBuf>, root: &Path) -> ClaudeCodeRunner {
    ClaudeCodeRunner::new(
        claude,
        PathBuf::from("/opt/lumeart/bin/lumeart"),
        root.join("session.json"),
        root.join("trabajo"),
    )
}

fn run(
    runner: &ClaudeCodeRunner,
    task: AgentTask,
    cancel: CancellationToken,
) -> (
    Result<lume_app::ports::RunSummary, PortError>,
    Vec<AgentEvent>,
) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let (sink, mut rx) = EventSink::channel();
    let result = rt.block_on(async {
        tokio::time::timeout(
            Duration::from_secs(30),
            runner.run(task, Arc::new(FakeTools::default()), sink, cancel),
        )
        .await
        .expect("el runner no termina")
    });
    let mut events = Vec::new();
    while let Ok(e) = rx.try_recv() {
        events.push(e);
    }
    (result, events)
}

// --- Traducción del stream ------------------------------------------------

/// El stream grabado da los eventos de la ejecución, en orden.
#[test]
fn the_recorded_stream_gives_the_right_events() {
    let mut parser = StreamParser::default();
    let events: Vec<AgentEvent> = SETA.lines().flat_map(|l| parser.line(l)).collect();
    let kinds: Vec<String> = events
        .iter()
        .map(|e| match e {
            AgentEvent::Text(_) => "texto".to_string(),
            AgentEvent::Thinking(_) => "piensa".to_string(),
            AgentEvent::ToolCall { name, .. } => format!("llama {name}"),
            AgentEvent::ToolResult { name, .. } => format!("resultado {name}"),
            AgentEvent::Usage(_) => "uso".to_string(),
        })
        .collect();
    assert_eq!(
        kinds,
        [
            "texto",
            "llama Bash",
            "resultado Bash",
            "llama Read",
            "resultado Read",
            "texto",
            "uso"
        ]
    );
    let AgentEvent::Text(plan) = &events[0] else {
        unreachable!()
    };
    assert!(plan.starts_with("**Composición**"), "{plan}");
    let AgentEvent::ToolCall { input_json, .. } = &events[1] else {
        unreachable!()
    };
    let input: serde_json::Value = serde_json::from_str(input_json).unwrap();
    assert!(
        input["command"]
            .as_str()
            .unwrap()
            .starts_with("pxcli batch '")
    );
    let AgentEvent::ToolResult { output, .. } = &events[2] else {
        unreachable!()
    };
    assert_eq!(
        output.content,
        vec![ContentPart::Text(
            "batch: 22 ok, 0 failed\nbatch: 32 ok, 0 failed\nok".into()
        )]
    );
    assert!(!output.is_error);
    // `Read` de la revisión: llega la imagen (PNG decodificado).
    let AgentEvent::ToolResult { output, .. } = &events[4] else {
        unreachable!()
    };
    let ContentPart::Image { media_type, data } = &output.content[0] else {
        panic!("se esperaba la imagen de la revisión: {:?}", output.content);
    };
    assert_eq!(media_type, "image/png");
    assert_eq!(&data[..8], b"\x89PNG\r\n\x1a\n");
    let AgentEvent::Usage(usage) = &events[6] else {
        unreachable!()
    };
    assert_eq!(
        (usage.input_tokens, usage.output_tokens, usage.cached_tokens),
        (4 + 6268, 1711, 10562)
    );

    assert_eq!(
        parser.session_id.as_deref(),
        Some("acf697c8-43f6-4a7a-8b62-8c5968810b2a")
    );
    assert_eq!(parser.model.as_deref(), Some("claude-sonnet-5-5"));
    let summary = parser.summary(RunEnd::Finished);
    assert_eq!(summary.steps, 3);
    assert!(
        summary
            .final_text
            .starts_with("He dibujado una seta de estilo RPG")
    );
}

#[test]
fn unknown_or_broken_lines_are_ignored() {
    let mut parser = StreamParser::default();
    assert!(parser.line("esto no es JSON").is_empty());
    assert!(
        parser
            .line(r#"{"type":"stream_event","event":{"type":"message_start"}}"#)
            .is_empty()
    );
    assert!(parser.line(r#"{"type":"rate_limit_event"}"#).is_empty());
    assert_eq!(parser.steps(), 0);
}

// --- Cómo se lanza ----------------------------------------------------------

/// Los argumentos de pxcli adaptados, y el entorno sin claves y con la
/// sesión, la ejecución y `lumeart` en el `PATH`.
#[test]
fn claude_is_launched_like_pxcli_without_api_keys() {
    let root = tempfile::tempdir().unwrap();
    let r = runner(Some("/usr/bin/claude".into()), root.path());
    let args = r.args(&task(4));
    let expected: Vec<&str> = vec![
        "-p",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--tools",
        "Bash,Read,Write",
        "--allowedTools",
        "Bash(lumeart:*)",
        "Read",
        "Edit(./notas.md)",
        "--strict-mcp-config",
        "--setting-sources",
        "project,local",
        "--append-system-prompt-file",
        SYSTEM_FILE,
        "--model",
        "sonnet",
    ];
    assert_eq!(args, expected);
    let mut no_model = task(4);
    no_model.model = " ".into();
    assert!(!r.args(&no_model).contains(&"--model".to_string()));

    let command = r.command(Path::new("/usr/bin/claude"), &task(4), root.path());
    let envs: Vec<(String, Option<String>)> = command
        .as_std()
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|v| v.to_string_lossy().into_owned()),
            )
        })
        .collect();
    for key in REMOVED_ENV {
        assert!(
            envs.contains(&(key.to_string(), None)),
            "{key} se quita: {envs:?}"
        );
    }
    let get = |k: &str| {
        envs.iter()
            .find(|(name, _)| name == k)
            .and_then(|(_, v)| v.clone())
    };
    assert_eq!(
        get("LUMEART_SESSION"),
        Some(
            root.path()
                .join("session.json")
                .to_string_lossy()
                .into_owned()
        )
    );
    assert_eq!(get("LUMEART_RUN").as_deref(), Some("4"));
    assert!(get("PATH").unwrap().starts_with("/opt/lumeart/bin"));
}

/// Sin `claude`, el error dice cómo instalarlo o elegir otro proveedor.
#[test]
fn without_claude_the_error_says_how_to_install_it() {
    let root = tempfile::tempdir().unwrap();
    let message = missing_claude().to_string();
    assert!(
        message.contains("https://claude.com/claude-code"),
        "{message}"
    );
    assert!(
        message.contains("npm install -g @anthropic-ai/claude-code"),
        "{message}"
    );
    assert!(
        message.contains("elige otro proveedor en Ajustes"),
        "{message}"
    );

    let (result, events) = run(
        &runner(None, root.path()),
        task(1),
        CancellationToken::new(),
    );
    assert_eq!(result.unwrap_err(), missing_claude());
    assert!(events.is_empty());
    // Una ruta que ya no existe: el mismo error.
    let gone = root.path().join("no-existe").join("claude");
    let (result, _) = run(
        &runner(Some(gone), root.path()),
        task(1),
        CancellationToken::new(),
    );
    assert_eq!(result.unwrap_err(), missing_claude());
}

// --- Con un `claude` falso (Unix) ------------------------------------------

#[cfg(unix)]
fn fake_claude(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("claude");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// El runner lanza el `claude` falso, le da el encargo por la entrada y
/// traduce su stream; deja la copia del stream y las instrucciones en la
/// carpeta de la ejecución.
#[cfg(unix)]
#[test]
fn a_fake_claude_run_is_translated() {
    let root = tempfile::tempdir().unwrap();
    let stream = root.path().join("seta.stream.jsonl");
    std::fs::write(&stream, SETA).unwrap();
    let claude = fake_claude(
        root.path(),
        &format!(
            r#"printf '%s\n' "$@" > args.txt
cat > prompt.txt
printf '%s|%s\n' "$LUMEART_SESSION" "$LUMEART_RUN" > env.txt
cat '{}'"#,
            stream.display()
        ),
    );
    let r = runner(Some(claude), root.path());
    let mut t = task(9);
    t.attachments = vec![Attachment {
        label: "Imagen de referencia".into(),
        media_type: "image/png".into(),
        data: b"PNG-REF".to_vec(),
    }];
    let (result, events) = run(&r, t.clone(), CancellationToken::new());
    let summary = result.unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::Finished, 3));
    assert!(summary.final_text.starts_with("He dibujado una seta"));
    assert_eq!(events.len(), 7);

    let dir = r.run_dir(&t);
    let read = |name: &str| std::fs::read_to_string(dir.join(name)).unwrap();
    assert!(read("args.txt").contains("Bash(lumeart:*)"));
    let prompt = read("prompt.txt");
    assert!(
        prompt.starts_with("Petición: una seta roja con puntos blancos"),
        "{prompt}"
    );
    assert!(prompt.contains("./referencia-1.png"), "{prompt}");
    assert_eq!(
        std::fs::read(dir.join("referencia-1.png")).unwrap(),
        b"PNG-REF"
    );
    assert!(read(SYSTEM_FILE).contains("lumeart batch"));
    assert_eq!(
        read("env.txt").trim(),
        format!("{}|9", root.path().join("session.json").display())
    );
    assert_eq!(read("stream.jsonl").lines().count(), SETA.lines().count());
}

/// Un `result` con error es un error claro; el límite de turnos de Claude
/// Code es el límite de pasos.
#[cfg(unix)]
#[test]
fn failed_results_and_limits() {
    let root = tempfile::tempdir().unwrap();
    let failed = fake_claude(
        root.path(),
        r#"cat > /dev/null
echo '{"type":"result","subtype":"error_during_execution","is_error":true,"num_turns":1,"result":"Credit balance is too low","usage":{}}'"#,
    );
    let (result, _) = run(
        &runner(Some(failed), root.path()),
        task(1),
        CancellationToken::new(),
    );
    assert_eq!(
        result.unwrap_err(),
        PortError::Provider("Claude Code falló: Credit balance is too low".into())
    );

    let silent = fake_claude(
        root.path(),
        "cat > /dev/null\necho 'algo fue mal' >&2\nexit 3",
    );
    let (result, _) = run(
        &runner(Some(silent), root.path()),
        task(1),
        CancellationToken::new(),
    );
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("claude terminó sin resultado") && err.contains("algo fue mal"),
        "{err}"
    );

    // El stream de la seta tiene 2 mensajes del modelo antes del resultado:
    // con 1 paso como máximo, se para en el segundo.
    let stream = root.path().join("seta.stream.jsonl");
    std::fs::write(&stream, SETA).unwrap();
    let seta = fake_claude(
        root.path(),
        &format!("cat > /dev/null\ncat '{}'\nsleep 30", stream.display()),
    );
    let mut t = task(2);
    t.limits.max_steps = 1;
    let (result, _) = run(
        &runner(Some(seta.clone()), root.path()),
        t,
        CancellationToken::new(),
    );
    let summary = result.unwrap();
    assert_eq!((summary.end, summary.steps), (RunEnd::StepLimit, 1));

    // Tokens: el primer mensaje ya pasa de 1000.
    let mut t = task(3);
    t.limits.max_tokens = 1000;
    let (result, _) = run(
        &runner(Some(seta), root.path()),
        t,
        CancellationToken::new(),
    );
    assert_eq!(result.unwrap().end, RunEnd::TokenLimit);
}

/// ¿Sigue vivo el proceso? (un zombi cuenta como muerto)
#[cfg(unix)]
fn alive(pid: u32) -> bool {
    if let Ok(stat) = std::fs::read_to_string(format!("/proc/{pid}/stat")) {
        let state = stat
            .rsplit(')')
            .next()
            .unwrap_or("")
            .trim_start()
            .chars()
            .next();
        return state != Some('Z');
    }
    if Path::new("/proc").exists() {
        return false;
    }
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Detener mata `claude` y lo que haya lanzado (su grupo de procesos).
#[cfg(unix)]
#[test]
fn stopping_kills_claude_and_its_children() {
    let root = tempfile::tempdir().unwrap();
    let claude = fake_claude(
        root.path(),
        r#"cat > /dev/null
sleep 300 &
echo $! > child.pid
echo $$ > parent.pid
sleep 300"#,
    );
    let r = runner(Some(claude), root.path());
    let t = task(5);
    let dir = r.run_dir(&t);
    let cancel = CancellationToken::new();
    let later = cancel.clone();
    let watch = dir.clone();
    std::thread::spawn(move || {
        for _ in 0..200 {
            if watch.join("parent.pid").exists() && watch.join("child.pid").exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_millis(100));
        later.cancel();
    });
    let started = std::time::Instant::now();
    let (result, _) = run(&r, t, cancel);
    assert_eq!(result.unwrap().end, RunEnd::Cancelled);
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "detener es inmediato"
    );

    let pid = |name: &str| -> u32 {
        std::fs::read_to_string(dir.join(name))
            .unwrap()
            .trim()
            .parse()
            .unwrap()
    };
    let (parent, child) = (pid("parent.pid"), pid("child.pid"));
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while (alive(parent) || alive(child)) && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(!alive(parent), "claude sigue vivo");
    assert!(!alive(child), "su hijo sigue vivo");
}

/// En el bucle: el lienzo de la vuelta queda en `lienzo.png` y lo que
/// Claude Code escribe en `./notas.md` pasa al documento con `write_notes`.
#[cfg(unix)]
#[test]
fn turn_canvas_and_notes_go_through_files() {
    let root = tempfile::tempdir().unwrap();
    let claude = fake_claude(
        root.path(),
        r#"cat > prompt.txt
printf 'Hice el contorno. Falta el brillo.' > notas.md
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"#,
    );
    let r = runner(Some(claude), root.path());
    let tools = Arc::new(FakeTools::default());
    let mut t = task(6);
    t.attachments = vec![Attachment {
        label: lume_app::ports::CANVAS_ATTACHMENT.into(),
        media_type: "image/png".into(),
        data: b"PNG-LIENZO".to_vec(),
    }];
    let rt = tokio::runtime::Runtime::new().unwrap();
    let summary = rt
        .block_on(r.run(
            t.clone(),
            tools.clone(),
            EventSink::channel().0,
            CancellationToken::new(),
        ))
        .unwrap();
    assert_eq!(summary.end, RunEnd::Finished);
    let dir = r.run_dir(&t);
    assert_eq!(
        std::fs::read(dir.join("lienzo.png")).unwrap(),
        b"PNG-LIENZO"
    );
    assert!(
        !dir.join("referencia-1.png").exists(),
        "el lienzo no es una referencia"
    );
    let prompt = std::fs::read_to_string(dir.join("prompt.txt")).unwrap();
    assert!(prompt.contains("./lienzo.png"), "{prompt}");
    let calls = tools.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "write_notes");
    assert!(calls[0].1.contains("Hice el contorno. Falta el brillo."));
}

/// Con `Resume`: el stream deja la sesión de Claude Code y la vuelta
/// siguiente la retoma con `--resume`.
#[test]
fn claude_code_sessions_can_be_resumed() {
    let mut parser = StreamParser::default();
    for line in SETA.lines() {
        parser.line(line);
    }
    let summary = parser.summary(RunEnd::Finished);
    assert_eq!(
        summary.conversation,
        Some(lume_app::ports::Conversation::ClaudeSession(
            "acf697c8-43f6-4a7a-8b62-8c5968810b2a".into()
        ))
    );
    assert_eq!(parser.summary(RunEnd::Cancelled).conversation, None);

    let root = tempfile::tempdir().unwrap();
    let r = runner(Some("/usr/bin/claude".into()), root.path());
    let mut t = task(2);
    assert!(!r.args(&t).contains(&"--resume".to_string()));
    t.resume = summary.conversation;
    let args = r.args(&t);
    let i = args.iter().position(|a| a == "--resume").unwrap();
    assert_eq!(args[i + 1], "acf697c8-43f6-4a7a-8b62-8c5968810b2a");
}

/// Un trabajo de animación: anim.js y base.png en la carpeta, permiso para
/// editar `./anim.js` y, si lo cambió, pasa a la sesión con `write_anim`.
#[cfg(unix)]
#[test]
fn an_animation_task_goes_through_anim_js() {
    let root = tempfile::tempdir().unwrap();
    let claude = fake_claude(
        root.path(),
        r#"cat > prompt.txt
test -f base.png && cp base.png vista.png
printf 'action("andar", { frames: 2 }, () => { base(); });\n' > anim.js
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"#,
    );
    let r = runner(Some(claude), root.path());
    let tools = Arc::new(FakeTools::default());
    let mut t = task(8);
    t.anim = Some(lume_app::ports::AnimTask {
        source: "action(\"andar\", { frames: 1 }, () => {});\n".into(),
        base_png: b"PNG-BASE".to_vec(),
    });
    let args = r.args(&t);
    assert!(args.iter().any(|a| a == "Edit(./anim.js)"), "{args:?}");
    // Y la herramienta `Edit`, para cambiar unas líneas sin reescribirlo.
    assert!(args.iter().any(|a| a == "Bash,Read,Write,Edit"), "{args:?}");
    let plain = r.args(&task(9));
    assert!(
        !plain.iter().any(|a| a == "Edit(./anim.js)"),
        "sin animación, no"
    );
    assert!(plain.iter().any(|a| a == "Bash,Read,Write"), "{plain:?}");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(r.run(
        t.clone(),
        tools.clone(),
        EventSink::channel().0,
        CancellationToken::new(),
    ))
    .unwrap();
    let dir = r.run_dir(&t);
    assert_eq!(std::fs::read(dir.join("vista.png")).unwrap(), b"PNG-BASE");
    let calls = tools.calls.lock().unwrap().clone();
    let write = calls
        .iter()
        .find(|(name, _)| name == "write_anim")
        .expect("write_anim");
    let input: serde_json::Value = serde_json::from_str(&write.1).unwrap();
    assert_eq!(
        input["source"],
        "action(\"andar\", { frames: 2 }, () => { base(); });\n"
    );

    // Si no lo cambia, no se toca la animación.
    let same = fake_claude(
        root.path(),
        r#"cat > prompt.txt
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"#,
    );
    let r = runner(Some(same), root.path());
    let tools = Arc::new(FakeTools::default());
    let mut t = task(10);
    t.anim = Some(lume_app::ports::AnimTask {
        source: "x".into(),
        base_png: Vec::new(),
    });
    rt.block_on(r.run(
        t,
        tools.clone(),
        EventSink::channel().0,
        CancellationToken::new(),
    ))
    .unwrap();
    assert!(
        !tools
            .calls
            .lock()
            .unwrap()
            .iter()
            .any(|(n, _)| n == "write_anim")
    );
}

/// Un trabajo vectorial: dibujo.js en la carpeta, permiso para editar
/// `./dibujo.js` y, cada vez que cambia, pasa a la sesión con `write_vec`
/// (una versión por cambio). Si no lo cambia, no se toca el documento.
#[cfg(unix)]
#[test]
fn a_vector_task_goes_through_dibujo_js() {
    let root = tempfile::tempdir().unwrap();
    let claude = fake_claude(
        root.path(),
        r##"cat > prompt.txt
cp dibujo.js antes.js
printf 'canvas({ width: 64, height: 64 })\ncircle(32, 32, 20, { fill: "#e8803a" })\n' > dibujo.js
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"##,
    );
    let r = runner(Some(claude), root.path());
    let tools = Arc::new(FakeTools::default());
    let mut t = task(11);
    t.vector = Some(lume_app::ports::VecTask {
        source: "canvas({ width: 64, height: 64 })\n".into(),
    });
    let args = r.args(&t);
    assert!(args.iter().any(|a| a == "Edit(./dibujo.js)"), "{args:?}");
    assert!(args.iter().any(|a| a == "Bash,Read,Write,Edit"), "{args:?}");
    let plain = r.args(&task(12));
    assert!(
        !plain.iter().any(|a| a == "Edit(./dibujo.js)"),
        "sin dibujo vectorial, no"
    );
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(r.run(
        t.clone(),
        tools.clone(),
        EventSink::channel().0,
        CancellationToken::new(),
    ))
    .unwrap();
    let dir = r.run_dir(&t);
    // El agente encontró el dibujo.js del documento.
    assert_eq!(
        std::fs::read_to_string(dir.join("antes.js")).unwrap(),
        "canvas({ width: 64, height: 64 })\n"
    );
    // El encargo es el del modo vector.
    let prompt = std::fs::read_to_string(dir.join("prompt.txt")).unwrap();
    assert!(prompt.contains("dibujo.js"), "{prompt}");
    let calls = tools.calls.lock().unwrap().clone();
    let writes: Vec<&(String, String)> = calls.iter().filter(|(n, _)| n == "write_vec").collect();
    assert_eq!(writes.len(), 1, "{calls:?}");
    let input: serde_json::Value = serde_json::from_str(&writes[0].1).unwrap();
    assert_eq!(
        input["source"],
        "canvas({ width: 64, height: 64 })\ncircle(32, 32, 20, { fill: \"#e8803a\" })\n"
    );

    // Si no lo cambia, no se llama a write_vec.
    let same = fake_claude(
        root.path(),
        r#"cat > prompt.txt
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"#,
    );
    let r = runner(Some(same), root.path());
    let tools = Arc::new(FakeTools::default());
    let mut t = task(13);
    t.vector = Some(lume_app::ports::VecTask { source: "x".into() });
    rt.block_on(r.run(
        t,
        tools.clone(),
        EventSink::channel().0,
        CancellationToken::new(),
    ))
    .unwrap();
    let calls = tools.calls.lock().unwrap().clone();
    assert!(!calls.iter().any(|(n, _)| n == "write_vec"), "{calls:?}");
}

/// Con el plugin oficial (`lumeart eval`, variante A): se carga con
/// `--plugin-dir`, el encargo va en `encargo.md` y a Claude le llega el
/// comando del plugin con la promesa y las vueltas; las vueltas se leen del
/// archivo de estado del plugin mientras trabaja. El crítico no lo usa.
#[cfg(unix)]
#[test]
fn the_official_plugin_runs_the_loop() {
    let root = tempfile::tempdir().unwrap();
    let claude = fake_claude(
        root.path(),
        r#"printf '%s\n' "$@" > args.txt
cat > prompt.txt
mkdir -p .claude
for n in 1 2 3; do
  printf -- '---\nactive: true\niteration: %s\nmax_iterations: 4\n---\nLee encargo.md\n' "$n" > .claude/ralph-loop.local.md
  echo '{"type":"system","subtype":"init"}'
  sleep 0.3
done
rm .claude/ralph-loop.local.md
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":3,"result":"Seta terminada. <promise>LISTO</promise>","usage":{}}'"#,
    );
    let plugin = lume_agent::RalphPlugin {
        dir: PathBuf::from("/plugins/ralph-loop"),
        max_iterations: 4,
    };
    let r = runner(Some(claude), root.path()).with_ralph(plugin.clone());
    let t = task(21);
    let args = r.args(&t);
    let i = args
        .iter()
        .position(|a| a == "--plugin-dir")
        .expect("--plugin-dir");
    assert_eq!(args[i + 1], "/plugins/ralph-loop");
    let (summary, _) = run(&r, t.clone(), CancellationToken::new());
    let summary = summary.unwrap();
    assert_eq!(summary.loops, Some(3));
    assert_eq!(summary.end, RunEnd::Finished);
    assert!(summary.final_text.ends_with("<promise>LISTO</promise>"));
    let dir = r.run_dir(&t);
    assert_eq!(
        std::fs::read_to_string(dir.join("prompt.txt")).unwrap(),
        "/ralph-loop:ralph-loop Lee encargo.md y haz lo que pide --completion-promise LISTO --max-iterations 4"
    );
    let task_text = std::fs::read_to_string(dir.join("encargo.md")).unwrap();
    assert!(
        task_text.contains("una seta roja con puntos blancos"),
        "{task_text}"
    );
    // El plugin no admite cualquier texto: el comando no lleva paréntesis
    // ni comillas.
    assert!(!plugin.prompt().contains(['(', ')', '"', '\'', '`']));
    assert_eq!(
        lume_agent::claude_code::ralph_iteration("---\niteration: 7\n---"),
        Some(7)
    );

    // El crítico, aunque el runner lo lleve, no carga el plugin.
    let critic = runner(None, root.path()).with_ralph(plugin).as_critic();
    assert!(!critic.args(&task(22)).iter().any(|a| a == "--plugin-dir"));
    // Sin plugin, no hay vueltas propias.
    let plain = fake_claude(
        root.path(),
        r#"cat > /dev/null
echo '{"type":"result","subtype":"success","is_error":false,"num_turns":1,"result":"ok","usage":{}}'"#,
    );
    let (summary, _) = run(
        &runner(Some(plain), root.path()),
        task(23),
        CancellationToken::new(),
    );
    assert_eq!(summary.unwrap().loops, None);
}
