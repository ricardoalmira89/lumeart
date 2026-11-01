//! `lumeart eval`: los mismos encargos con el plugin oficial (A) y con el
//! bucle de LumeArt (B: contexto vacío; C: misma conversación), una
//! comparación ciega por parejas y el informe.
//!
//! ```text
//! evaluacion/
//!   encargos.json            los encargos de esta evaluación
//!   imagenes/k3f9a2.png      el resultado de cada ejecución (por su clave)
//!   comparar/par-01.png      cada par: 1 a la izquierda, 2 a la derecha
//!   comparar/pares.json      los pares (solo claves)
//!   comparar/respuestas.json lo que se eligió
//!   informe.md               al terminar la comparación
//!   .datos/resultados.json   métricas y qué variante es cada clave
//!   trabajo/                 carpetas de Claude Code
//! ```
//!
//! Nada de lo que se ve antes de terminar la comparación dice qué variante
//! hizo qué: ni la salida de `run`, ni los nombres de las imágenes, ni los
//! pares. Las variantes solo están en `.datos/`.

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use lume_app::eval::{EvalAgents, EvalRunner, keys};
use lume_app::ports::{BoxFuture, CancellationToken, PortError};
use lume_app::{ArtAgent, ArtAgents, DocumentStore, ProviderKind, SessionHandle};
use lume_cli::{EvalArgs, EvalCommand, EvalReportArgs, EvalRunArgs};
use lume_domain::eval::{
    Answer, BlindPair, Choice, EvalTask, RunMetrics, Variant, check_tasks, plan_pairs, report,
    reveal, side_by_side, tally,
};
use lume_domain::{Canvas, RunId};
use lume_store::FsStore;

use crate::mcp::JobAgents;

/// Los 20 encargos de LumeArt (de `reference/runs/`).
pub const DEFAULT_TASKS: &str = include_str!("../eval/encargos.json");

/// Ancho aproximado de las imágenes de los pares.
const PAIR_WIDTH: u32 = 640;

/// Los archivos de una evaluación.
#[derive(Debug, Clone)]
pub struct Layout {
    root: PathBuf,
}

impl Layout {
    pub fn new(root: impl Into<PathBuf>) -> Layout {
        Layout { root: root.into() }
    }
    fn tasks(&self) -> PathBuf {
        self.root.join("encargos.json")
    }
    fn data(&self) -> PathBuf {
        self.root.join(".datos")
    }
    fn results(&self) -> PathBuf {
        self.data().join("resultados.json")
    }
    fn meta(&self) -> PathBuf {
        self.data().join("evaluacion.json")
    }
    fn images(&self) -> PathBuf {
        self.root.join("imagenes")
    }
    fn image(&self, key: &str) -> PathBuf {
        self.images().join(format!("{key}.png"))
    }
    fn compare(&self) -> PathBuf {
        self.root.join("comparar")
    }
    fn pairs(&self) -> PathBuf {
        self.compare().join("pares.json")
    }
    fn answers(&self) -> PathBuf {
        self.compare().join("respuestas.json")
    }
    fn pair_image(&self, number: u32) -> PathBuf {
        self.compare().join(format!("par-{number:02}.png"))
    }
    fn report(&self) -> PathBuf {
        self.root.join("informe.md")
    }
    fn work(&self) -> PathBuf {
        self.root.join("trabajo")
    }
}

pub fn run(
    args: &EvalArgs,
    stdin: &mut dyn BufRead,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> i32 {
    let result = match &args.command {
        EvalCommand::Run(a) => run_tasks(a, stdout),
        EvalCommand::Compare(a) => compare(&Layout::new(&a.dir), stdin, stdout),
        EvalCommand::Report(a) => write_report(a, stdout),
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            let _ = writeln!(stderr, "Error: {e}");
            1
        }
    }
}

// --- Archivos ---------------------------------------------------------------

fn io(path: &Path) -> impl Fn(std::io::Error) -> String + '_ {
    move |e| format!("{}: {e}", path.display())
}

fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| format!("{}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(io(path)(e)),
    }
}

fn write_json<T: serde::Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(io(dir))?;
    }
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, text + "\n").map_err(io(path))
}

/// Los encargos de un JSON (una lista de `{id, prompt, width, height}`).
pub fn parse_tasks(text: &str) -> Result<Vec<EvalTask>, String> {
    let tasks: Vec<EvalTask> =
        serde_json::from_str(text).map_err(|e| format!("encargos no válidos: {e}"))?;
    check_tasks(&tasks)?;
    Ok(tasks)
}

fn load_results(layout: &Layout) -> Result<Vec<RunMetrics>, String> {
    Ok(read_json(&layout.results())?.unwrap_or_default())
}

fn load_tasks_of(layout: &Layout) -> Result<Vec<EvalTask>, String> {
    read_json(&layout.tasks())?.ok_or_else(|| {
        format!(
            "{} no es una evaluación: hazla antes con lumeart eval run -o {}",
            layout.root.display(),
            layout.root.display()
        )
    })
}

// --- eval run ---------------------------------------------------------------

/// El plugin oficial: el que se indica o el instalado en Claude Code
/// (`~/.claude/plugins/installed_plugins.json`).
pub fn find_plugin(given: &str) -> Result<PathBuf, String> {
    let is_plugin = |dir: &Path| dir.join(".claude-plugin").join("plugin.json").is_file();
    if !given.is_empty() {
        let dir = PathBuf::from(given);
        return if is_plugin(&dir) {
            Ok(dir)
        } else {
            Err(format!(
                "{given} no es la carpeta de un plugin de Claude Code (falta .claude-plugin/plugin.json)"
            ))
        };
    }
    let config = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| directories::BaseDirs::new().map(|d| d.home_dir().join(".claude")));
    let installed = config
        .map(|c| c.join("plugins").join("installed_plugins.json"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok());
    if let Some(plugins) = installed
        .as_ref()
        .and_then(|v| v.get("plugins"))
        .and_then(|p| p.as_object())
    {
        for (name, entries) in plugins {
            if !(name.starts_with("ralph-loop@") || name.starts_with("ralph-wiggum@")) {
                continue;
            }
            let found = entries
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|e| e.get("installPath").and_then(|p| p.as_str()))
                .map(PathBuf::from)
                .find(|p| is_plugin(p));
            if let Some(dir) = found {
                return Ok(dir);
            }
        }
    }
    Err("no encuentro el plugin oficial ralph-loop: instálalo en Claude Code (/plugin), indica su carpeta con --plugin o deja fuera la A con --variants B,C".into())
}

/// Los agentes de cada variante: los de los encargos de LumeArt (una API
/// local por sesión, para que Claude Code use `lumeart`), y en A con el
/// plugin oficial.
struct Agents {
    plain: JobAgents,
    plugin: Option<JobAgents>,
}

impl EvalAgents for Agents {
    fn agent(
        &self,
        session: SessionHandle,
        variant: Variant,
        run: RunId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        match (variant, &self.plugin) {
            (Variant::A, Some(plugin)) => plugin.agent(session, run),
            (Variant::A, None) => Box::pin(async {
                Err(PortError::Unavailable(
                    "falta el plugin oficial (variante A)".into(),
                ))
            }),
            _ => self.plain.agent(session, run),
        }
    }
}

fn run_tasks(a: &EvalRunArgs, out: &mut dyn Write) -> Result<(), String> {
    let text = if a.tasks.is_empty() {
        DEFAULT_TASKS.to_string()
    } else {
        std::fs::read_to_string(&a.tasks).map_err(io(Path::new(&a.tasks)))?
    };
    let mut tasks = parse_tasks(&text)?;
    if a.limit > 0 {
        tasks.truncate(a.limit);
    }
    let variants = Variant::parse_list(&a.variants)?;
    let mut settings = crate::daemon::settings_store()
        .load()
        .map_err(|e| e.to_string())?;
    if a.turns > 0 {
        settings.max_turns = a.turns;
    }
    if !a.model.trim().is_empty() {
        let model = a.model.trim().to_string();
        match settings.provider {
            ProviderKind::ClaudeCode => settings.claude_code_model = model,
            ProviderKind::Anthropic => settings.anthropic_model = model,
            ProviderKind::OpenAi => settings.openai_model = model,
            ProviderKind::Local => {}
        }
    }
    let settings = settings.validated();
    let ralph = if variants.contains(&Variant::A) {
        if settings.provider != ProviderKind::ClaudeCode {
            return Err("la variante A (plugin oficial) solo funciona con Claude Code: elígelo en Ajustes o usa --variants B,C".into());
        }
        Some(lume_agent::RalphPlugin {
            dir: find_plugin(&a.plugin)?,
            max_iterations: settings.max_turns,
        })
    } else {
        None
    };
    let layout = Layout::new(&a.out);
    std::fs::create_dir_all(&layout.root).map_err(io(&layout.root))?;
    // La semilla: la de esta evaluación si ya existe.
    let seed = match read_json::<serde_json::Value>(&layout.meta())?
        .and_then(|m| m.get("seed").and_then(|s| s.as_u64()))
    {
        Some(seed) => seed,
        None => {
            let seed = if a.seed != 0 {
                a.seed
            } else {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_nanos() as u64)
                    .unwrap_or(1)
            };
            write_json(&layout.meta(), &serde_json::json!({ "seed": seed }))?;
            seed
        }
    };
    let store: Arc<dyn DocumentStore> = Arc::new(FsStore);
    let jobs = |ralph: Option<lume_agent::RalphPlugin>| {
        let mut agents = crate::mcp::job_agents(store.clone(), layout.work());
        agents.ralph = ralph;
        agents
    };
    let runner = EvalRunner {
        agents: Arc::new(Agents {
            plain: jobs(None),
            plugin: ralph.map(|p| jobs(Some(p))),
        }),
        store: store.clone(),
        settings,
        critic: !a.no_critic,
    };
    evaluate(&runner, &tasks, &variants, &layout, seed, out)?;
    Ok(())
}

/// El núcleo de `eval run`: hace lo que falte (si la evaluación se cortó,
/// sigue donde se quedó), guarda las imágenes con su clave y prepara la
/// comparación ciega. Lo que escribe en `out` no dice qué variante es cuál.
pub fn evaluate(
    runner: &EvalRunner,
    tasks: &[EvalTask],
    variants: &[Variant],
    layout: &Layout,
    seed: u64,
    out: &mut dyn Write,
) -> Result<Vec<RunMetrics>, String> {
    write_json(&layout.tasks(), &tasks)?;
    std::fs::create_dir_all(layout.images()).map_err(io(&layout.images()))?;
    let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
    let store = FsStore;
    let mut results = load_results(layout)?;
    let all_keys = keys(seed, tasks.len() * Variant::ALL.len());
    let _ = writeln!(
        out,
        "Evaluación: {} encargos, {} ejecuciones cada uno. Las imágenes van en {} con nombres que no dicen la variante.",
        tasks.len(),
        variants.len(),
        layout.images().display()
    );
    for (ti, task) in tasks.iter().enumerate() {
        let mut failed = 0;
        for (n, &variant) in variants.iter().enumerate() {
            let vi = Variant::ALL.iter().position(|v| *v == variant).unwrap_or(0);
            let done = results
                .iter()
                .any(|r| r.task == task.id && r.variant == variant && r.error.is_none());
            if done && layout.image(&all_keys[ti * 3 + vi]).is_file() {
                continue;
            }
            let index = ti * Variant::ALL.len() + vi;
            let key = all_keys[index].clone();
            let outcome = runtime.block_on(runner.run(
                task,
                variant,
                1000 + index as RunId,
                key.clone(),
                CancellationToken::new(),
            ));
            let png = store
                .encode_png(&outcome.canvas, 1)
                .map_err(|e| e.message)?;
            std::fs::write(layout.image(&key), png).map_err(io(&layout.image(&key)))?;
            if outcome.metrics.error.is_some() {
                failed += 1;
            }
            results.retain(|r| !(r.task == task.id && r.variant == variant));
            results.push(outcome.metrics);
            write_json(&layout.results(), &results)?;
            let _ = writeln!(
                out,
                "{}/{} · {}: {} de {} hechas{}",
                ti + 1,
                tasks.len(),
                task.id,
                n + 1,
                variants.len(),
                if failed > 0 { " (alguna falló)" } else { "" }
            );
        }
    }
    let pairs = prepare_comparison(layout, &results, seed)?;
    let _ = writeln!(
        out,
        "Hecho. Compara los {} pares a ciegas con: lumeart eval compare {}",
        pairs,
        layout.root.display()
    );
    Ok(results)
}

/// Escribe los pares de la comparación ciega y sus imágenes. Si los pares
/// cambian (hay ejecuciones nuevas), las respuestas anteriores se borran.
fn prepare_comparison(layout: &Layout, runs: &[RunMetrics], seed: u64) -> Result<usize, String> {
    let pairs = plan_pairs(runs, seed);
    let before: Option<Vec<BlindPair>> = read_json(&layout.pairs())?;
    if before.as_ref() != Some(&pairs) {
        let _ = std::fs::remove_file(layout.answers());
    }
    std::fs::create_dir_all(layout.compare()).map_err(io(&layout.compare()))?;
    write_json(&layout.pairs(), &pairs)?;
    let store = FsStore;
    for pair in &pairs {
        let read = |key: &str| -> Result<Canvas, String> {
            store.read_png(&layout.image(key)).map_err(|e| e.message)
        };
        let both = side_by_side(&read(&pair.left)?, &read(&pair.right)?);
        let scale = (PAIR_WIDTH / both.width().max(1)).clamp(1, 16);
        let png = store.encode_png(&both, scale).map_err(|e| e.message)?;
        let path = layout.pair_image(pair.number);
        std::fs::write(&path, png).map_err(io(&path))?;
    }
    Ok(pairs.len())
}

// --- eval compare -----------------------------------------------------------

/// Comparación ciega: pregunta por cada par sin respuesta y, cuando están
/// todos, revela las variantes y escribe el informe. Se puede dejar a medias
/// (con `s` o al cerrar la entrada) y seguir otro día.
pub fn compare(
    layout: &Layout,
    stdin: &mut dyn BufRead,
    out: &mut dyn Write,
) -> Result<(), String> {
    let tasks = load_tasks_of(layout)?;
    let runs = load_results(layout)?;
    let pairs: Vec<BlindPair> = read_json(&layout.pairs())?.unwrap_or_default();
    if pairs.is_empty() {
        return Err(format!(
            "no hay pares que comparar: haz antes lumeart eval run -o {}",
            layout.root.display()
        ));
    }
    let mut answers: Vec<Answer> = read_json(&layout.answers())?.unwrap_or_default();
    let pending: Vec<&BlindPair> = pairs
        .iter()
        .filter(|p| !answers.iter().any(|a| a.pair == p.number))
        .collect();
    let w = |out: &mut dyn Write, text: String| {
        let _ = writeln!(out, "{text}");
    };
    if !pending.is_empty() {
        w(
            out,
            format!(
                "Comparación ciega: {} pares (faltan {}). Abre la imagen de cada par: a la izquierda está la 1 y a la derecha la 2. Escribe 1, 2 o = (empate); s para dejarlo y seguir otro día.",
                pairs.len(),
                pending.len()
            ),
        );
    }
    for pair in pending {
        let prompt = tasks
            .iter()
            .find(|t| t.id == pair.task)
            .map(|t| short(&t.prompt, 90))
            .unwrap_or_default();
        w(
            out,
            format!(
                "\nPar {} de {} · {}: «{prompt}»\n  {}",
                pair.number,
                pairs.len(),
                pair.task,
                layout.pair_image(pair.number).display()
            ),
        );
        loop {
            let _ = write!(out, "¿Cuál es mejor? [1/2/=/s] ");
            let _ = out.flush();
            let mut line = String::new();
            let read = stdin.read_line(&mut line).map_err(|e| e.to_string())?;
            let left = pairs.len() - answers.len();
            if read == 0 || line.trim().eq_ignore_ascii_case("s") {
                w(
                    out,
                    format!(
                        "\nGuardado. Faltan {left} pares: sigue con lumeart eval compare {}",
                        layout.root.display()
                    ),
                );
                return Ok(());
            }
            match Choice::parse(&line) {
                Some(choice) => {
                    answers.push(Answer {
                        pair: pair.number,
                        choice,
                    });
                    write_json(&layout.answers(), &answers)?;
                    break;
                }
                None => w(out, "Escribe 1, 2 o =.".into()),
            }
        }
    }
    // Todas respondidas: ahora sí, qué era cada cosa.
    w(out, "\nYa están todos. Esto era cada par:".into());
    for line in reveal(&runs, &pairs, &answers) {
        w(out, format!("  {line}"));
    }
    let prefs = tally(&runs, &pairs, &answers);
    w(out, "\nPreferencias:".into());
    for (variant, [won, lost, tie]) in &prefs.by_variant {
        w(
            out,
            format!(
                "  {}: preferida {won}, no preferida {lost}, empates {tie}",
                variant.label()
            ),
        );
    }
    let text = report(&tasks, &runs, Some((&pairs, &answers)));
    std::fs::write(layout.report(), &text).map_err(io(&layout.report()))?;
    w(out, format!("\nInforme: {}", layout.report().display()));
    Ok(())
}

fn short(text: &str, max: usize) -> String {
    let line = text.lines().next().unwrap_or_default().trim();
    if line.chars().count() <= max {
        line.to_string()
    } else {
        let cut: String = line.chars().take(max).collect();
        format!("{}…", cut.trim_end())
    }
}

// --- eval report ------------------------------------------------------------

fn write_report(a: &EvalReportArgs, out: &mut dyn Write) -> Result<(), String> {
    let layout = Layout::new(&a.dir);
    let tasks = load_tasks_of(&layout)?;
    let runs = load_results(&layout)?;
    let pairs: Vec<BlindPair> = read_json(&layout.pairs())?.unwrap_or_default();
    let answers: Vec<Answer> = read_json(&layout.answers())?.unwrap_or_default();
    let missing = pairs
        .iter()
        .filter(|p| !answers.iter().any(|x| x.pair == p.number))
        .count();
    if missing > 0 && !a.force {
        return Err(format!(
            "faltan {missing} de {} pares de la comparación ciega y el informe dice qué variante es cada una: termínala con lumeart eval compare {} o usa --force",
            pairs.len(),
            layout.root.display()
        ));
    }
    let blind = (!pairs.is_empty()).then_some((pairs.as_slice(), answers.as_slice()));
    let text = report(&tasks, &runs, blind);
    std::fs::write(layout.report(), &text).map_err(io(&layout.report()))?;
    let _ = write!(out, "{text}");
    let _ = writeln!(out, "\nInforme: {}", layout.report().display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::ports::{
        AgentRunner, AgentTask, EventSink, PreparedAgent, RunEnd, RunSummary, ToolBox, Usage,
    };
    use lume_app::{AgentBackend, Settings};
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// Artista de mentira: pinta un cuadrado del color de su variante y
    /// promete; el crítico ve un problema bajo.
    struct Painter {
        color: &'static str,
        critic: bool,
        calls: Arc<AtomicUsize>,
    }

    impl AgentRunner for Painter {
        fn run<'a>(
            &'a self,
            task: AgentTask,
            tools: ToolBox,
            _events: EventSink,
            _cancel: CancellationToken,
        ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
            Box::pin(async move {
                let final_text = if self.critic {
                    "```json\n{\"aprobado\": true, \"problemas\": [{\"gravedad\": \"baja\", \"problema\": \"x\"}]}\n```".to_string()
                } else {
                    self.calls.fetch_add(1, Ordering::SeqCst);
                    let (w, h) = task.canvas;
                    let input = serde_json::json!({
                        "commands": format!("fill_rect 1 1 {} {} {}", w - 2, h - 2, self.color)
                    });
                    tools.call("draw", &input.to_string()).await;
                    "Hecho. <promise>LISTO</promise>".to_string()
                };
                Ok(RunSummary {
                    steps: 1,
                    usage: Usage {
                        input_tokens: 10,
                        output_tokens: 5,
                        cached_tokens: 0,
                    },
                    final_text,
                    end: RunEnd::Finished,
                    conversation: None,
                    loops: Some(2),
                })
            })
        }
    }

    struct Backend {
        tools: ToolBox,
        color: &'static str,
        calls: Arc<AtomicUsize>,
    }

    impl AgentBackend for Backend {
        fn prepare(&self, _: &Settings, _: RunId) -> Result<PreparedAgent, PortError> {
            Ok(PreparedAgent {
                runner: Arc::new(Painter {
                    color: self.color,
                    critic: false,
                    calls: self.calls.clone(),
                }),
                tools: self.tools.clone(),
                model: "m".into(),
            })
        }

        fn prepare_critic(&self, _: &Settings, _: RunId) -> Result<PreparedAgent, PortError> {
            Ok(PreparedAgent {
                runner: Arc::new(Painter {
                    color: self.color,
                    critic: true,
                    calls: self.calls.clone(),
                }),
                tools: self.tools.clone(),
                model: "m".into(),
            })
        }
    }

    struct FakeAgents {
        calls: Arc<AtomicUsize>,
    }

    impl EvalAgents for FakeAgents {
        fn agent(
            &self,
            session: SessionHandle,
            variant: Variant,
            run: RunId,
        ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
            let calls = self.calls.clone();
            Box::pin(async move {
                let tools: ToolBox = Arc::new(lume_agent::SessionTools::new(
                    session,
                    Arc::new(lume_app::testing::MemoryStore::new()),
                    run,
                ));
                let color = match variant {
                    Variant::A => "#cc2222",
                    Variant::B => "#22cc22",
                    Variant::C => "#2222cc",
                };
                Ok(ArtAgent {
                    backend: Arc::new(Backend {
                        tools,
                        color,
                        calls,
                    }),
                    guard: Box::new(()),
                })
            })
        }
    }

    fn tasks() -> Vec<EvalTask> {
        let t = |id: &str, prompt: &str, w| EvalTask {
            id: id.into(),
            prompt: prompt.into(),
            width: w,
            height: 16,
            origin: String::new(),
        };
        vec![
            t("seta", "una seta roja", 16),
            t("cofre", "un cofre del tesoro", 24),
        ]
    }

    fn runner(calls: Arc<AtomicUsize>) -> EvalRunner {
        EvalRunner {
            agents: Arc::new(FakeAgents { calls }),
            store: Arc::new(FsStore),
            settings: Settings::default(),
            critic: true,
        }
    }

    /// Lo que se ve antes de terminar la comparación no nombra variantes.
    fn assert_blind(text: &str) {
        for word in [
            "A (",
            "B (",
            "C (",
            "= A",
            "= B",
            "= C",
            "plugin",
            "contexto vacío",
            "misma conversación",
            "variante A",
            "variante B",
            "variante C",
        ] {
            assert!(!text.contains(word), "revela «{word}»:\n{text}");
        }
    }

    /// `run` hace los encargos con las tres variantes y deja la comparación
    /// lista sin decir cuál es cuál; si se vuelve a lanzar, no repite nada.
    #[test]
    fn run_prepares_a_blind_comparison() {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout::new(dir.path().join("evaluacion"));
        let calls = Arc::new(AtomicUsize::new(0));
        let mut out = Vec::new();
        let runs = evaluate(
            &runner(calls.clone()),
            &tasks(),
            &Variant::ALL,
            &layout,
            7,
            &mut out,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_blind(&text);
        assert!(text.contains("2/2 · cofre: 3 de 3 hechas"), "{text}");
        assert!(text.contains("Compara los 6 pares a ciegas"), "{text}");
        assert_eq!(runs.len(), 6);
        assert_eq!(calls.load(Ordering::SeqCst), 6);
        // Métricas de cada ejecución, con la nota del crítico.
        let a = runs.iter().find(|r| r.variant == Variant::A).unwrap();
        assert_eq!((a.turns, a.tokens, a.critic), (2, 15, Some(9.5)));
        assert!(a.passed);
        // Imágenes por clave (sin la variante en el nombre) y pares.
        for r in &runs {
            assert!(r.key.starts_with('k') && r.key.len() == 6, "{}", r.key);
            let png = layout.image(&r.key);
            let canvas = FsStore.read_png(&png).unwrap();
            assert_eq!(canvas.width(), if r.task == "seta" { 16 } else { 24 });
        }
        let pairs: Vec<BlindPair> = read_json(&layout.pairs()).unwrap().unwrap();
        assert_eq!(pairs.len(), 6);
        let pairs_text = std::fs::read_to_string(layout.pairs()).unwrap();
        assert_blind(&pairs_text);
        assert!(!pairs_text.contains("\"variant\""), "{pairs_text}");
        // El par: las dos, ampliadas (el ancho va en la cabecera del PNG).
        let png = std::fs::read(layout.pair_image(1)).unwrap();
        let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
        assert!(width >= PAIR_WIDTH / 2, "{width}");
        // Los nombres de las imágenes no dicen la variante.
        for entry in std::fs::read_dir(layout.images()).unwrap() {
            let name = entry.unwrap().file_name().to_string_lossy().into_owned();
            assert!(name.starts_with('k'), "{name}");
        }

        // Otra vez: ya está todo hecho.
        let mut out = Vec::new();
        evaluate(
            &runner(calls.clone()),
            &tasks(),
            &Variant::ALL,
            &layout,
            7,
            &mut out,
        )
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 6, "no repite ejecuciones");
    }

    /// La comparación pregunta par a par sin decir las variantes, se puede
    /// dejar a medias y, al terminar, las revela y escribe el informe.
    #[test]
    fn the_comparison_reveals_the_variants_only_at_the_end() {
        let dir = tempfile::tempdir().unwrap();
        let layout = Layout::new(dir.path().join("evaluacion"));
        let calls = Arc::new(AtomicUsize::new(0));
        evaluate(
            &runner(calls),
            &tasks(),
            &Variant::ALL,
            &layout,
            3,
            &mut Vec::new(),
        )
        .unwrap();

        // Dos respuestas (una no válida antes) y lo deja.
        let mut input: &[u8] = b"1\nquizas\n=\ns\n";
        let mut out = Vec::new();
        compare(&layout, &mut input, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_blind(&text);
        assert!(text.contains("Par 1 de 6 · "), "{text}");
        assert!(text.contains("par-01.png"), "{text}");
        assert!(text.contains("Escribe 1, 2 o =."), "{text}");
        assert!(text.contains("Faltan 4 pares"), "{text}");
        assert!(!layout.report().exists());
        let answers: Vec<Answer> = read_json(&layout.answers()).unwrap().unwrap();
        assert_eq!(answers.len(), 2);

        // El informe aún no: revelaría las variantes.
        let args = EvalReportArgs {
            dir: layout.root.to_string_lossy().into_owned(),
            force: false,
        };
        let err = write_report(&args, &mut Vec::new()).unwrap_err();
        assert!(err.contains("faltan 4 de 6 pares"), "{err}");

        // Sigue donde lo dejó; se cierra la entrada a medias.
        let mut input: &[u8] = b"2\n";
        let mut out = Vec::new();
        compare(&layout, &mut input, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_blind(&text);
        assert!(text.contains("faltan 4"), "{text}");
        assert!(text.contains("Faltan 3 pares"), "{text}");

        // Termina: ahora sí se revelan.
        let mut input: &[u8] = b"1\n2\n1\n";
        let mut out = Vec::new();
        compare(&layout, &mut input, &mut out).unwrap();
        let text = String::from_utf8(out).unwrap();
        let (before, after) = text.split_once("Ya están todos").unwrap();
        assert_blind(before);
        assert!(after.contains("Par 1 ("), "{after}");
        assert!(after.contains("A (plugin oficial): preferida"), "{after}");
        let report = std::fs::read_to_string(layout.report()).unwrap();
        assert!(report.contains("| Variante | Encargos | Nota del crítico | Pasa las comprobaciones | Vueltas | Tokens | Tiempo |"), "{report}");
        assert!(report.contains("6 de 6 pares respondidos."), "{report}");

        // Y el informe ya se puede pedir.
        let mut out = Vec::new();
        write_report(&args, &mut out).unwrap();
        assert!(
            String::from_utf8(out)
                .unwrap()
                .contains("## Comparación ciega")
        );
    }

    #[test]
    fn the_default_tasks_are_valid() {
        let tasks = parse_tasks(DEFAULT_TASKS).unwrap();
        assert_eq!(tasks.len(), 20);
        assert!(tasks.iter().all(|t| !t.origin.is_empty()));
        assert!(parse_tasks("[]").is_err());
        assert!(parse_tasks("{").is_err());
    }

    #[test]
    fn the_plugin_is_found_or_explained() {
        let dir = tempfile::tempdir().unwrap();
        let plugin = dir.path().join("ralph-loop");
        std::fs::create_dir_all(plugin.join(".claude-plugin")).unwrap();
        std::fs::write(plugin.join(".claude-plugin/plugin.json"), "{}").unwrap();
        assert_eq!(find_plugin(&plugin.to_string_lossy()).unwrap(), plugin);
        let err = find_plugin(&dir.path().to_string_lossy()).unwrap_err();
        assert!(err.contains("no es la carpeta de un plugin"), "{err}");
    }
}
