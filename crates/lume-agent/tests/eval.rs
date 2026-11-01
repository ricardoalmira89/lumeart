//! `EvalRunner` con el agente de verdad (`ApiRunner` + herramientas sobre la
//! sesión) y respuestas grabadas: cada variante hace el encargo en una
//! sesión nueva, se miden vueltas, tokens, tiempo y comprobaciones, y el
//! crítico pone nota al resultado.

use std::sync::{Arc, Mutex};

use lume_agent::{ApiRunner, SessionTools};
use lume_app::ports::{
    AgentRunner, AgentTask, BoxFuture, CancellationToken, ContentPart, EventSink, ModelResponse,
    PortError, RunSummary, StopReason, ToolBox, Usage,
};
use lume_app::testing::{FakeBackend, FakeProvider, MemoryStore};
use lume_app::{ArtAgent, EvalAgents, EvalRunner, SessionHandle, Settings};
use lume_domain::eval::{EvalTask, Variant};
use lume_domain::{Rgba, RunId};

fn reply(content: Vec<ContentPart>) -> ModelResponse {
    let stop = if content
        .iter()
        .any(|p| matches!(p, ContentPart::ToolUse { .. }))
    {
        StopReason::ToolUse
    } else {
        StopReason::EndTurn
    };
    ModelResponse {
        content,
        stop,
        usage: Usage {
            input_tokens: 100,
            output_tokens: 20,
            cached_tokens: 5,
        },
    }
}

fn draw(id: &str, commands: &str) -> ContentPart {
    ContentPart::ToolUse {
        id: id.into(),
        name: "draw".into(),
        input_json: serde_json::json!({ "commands": commands }).to_string(),
    }
}

fn text(t: &str) -> ContentPart {
    ContentPart::Text(t.into())
}

/// Veredicto del crítico con un problema medio y uno bajo: nota 8.
fn verdict() -> ModelResponse {
    reply(vec![text(
        "```json\n{\"aprobado\": true, \"problemas\": [{\"gravedad\": \"media\", \"problema\": \"el tallo es corto\"}, {\"gravedad\": \"baja\", \"problema\": \"un píxel suelto\"}]}\n```",
    )])
}

/// El runner del plugin oficial, de mentira: dibuja con las herramientas y
/// dice que dio tres vueltas (las que contaría el Stop hook).
struct PluginRunner;

impl AgentRunner for PluginRunner {
    fn run<'a>(
        &'a self,
        task: AgentTask,
        tools: ToolBox,
        _events: EventSink,
        _cancel: CancellationToken,
    ) -> BoxFuture<'a, Result<RunSummary, PortError>> {
        Box::pin(async move {
            assert_eq!(task.goal, "una seta roja");
            assert_eq!(task.canvas, (8, 8));
            let input = serde_json::json!({ "commands": "fill_rect 2 2 4 4 #cc2222" });
            let out = tools.call("draw", &input.to_string()).await;
            assert!(!out.is_error, "{out:?}");
            Ok(RunSummary {
                steps: 9,
                usage: Usage {
                    input_tokens: 3000,
                    output_tokens: 400,
                    cached_tokens: 9000,
                },
                final_text: "Seta lista. <promise>LISTO</promise>".into(),
                end: lume_app::ports::RunEnd::Finished,
                conversation: None,
                loops: Some(3),
            })
        })
    }
}

/// Los agentes de cada variante: A con el runner del plugin; B y C con el
/// `ApiRunner` sobre sus respuestas. El crítico usa el `ApiRunner` (con su
/// veredicto al final de las respuestas).
struct Agents {
    providers: Mutex<Vec<(Variant, Arc<FakeProvider>)>>,
    critic: Arc<FakeProvider>,
}

impl EvalAgents for Agents {
    fn agent(
        &self,
        session: SessionHandle,
        variant: Variant,
        run: RunId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>> {
        Box::pin(async move {
            let tools: ToolBox = Arc::new(SessionTools::new(
                session,
                Arc::new(MemoryStore::new()),
                run,
            ));
            let backend: Arc<dyn lume_app::AgentBackend> = match variant {
                Variant::A => Arc::new(Critiqued {
                    artist: Arc::new(PluginRunner),
                    critic: Arc::new(ApiRunner::new(self.critic.clone())),
                    tools,
                }),
                _ => {
                    let provider = self
                        .providers
                        .lock()
                        .unwrap()
                        .iter()
                        .find(|(v, _)| *v == variant)
                        .map(|(_, p)| p.clone())
                        .ok_or_else(|| PortError::Unavailable("sin agente".into()))?;
                    Arc::new(FakeBackend::new(Arc::new(ApiRunner::new(provider)), tools))
                }
            };
            Ok(ArtAgent {
                backend,
                guard: Box::new(()),
            })
        })
    }
}

/// Backend con un runner para el artista y otro para el crítico.
struct Critiqued {
    artist: Arc<dyn AgentRunner>,
    critic: Arc<dyn AgentRunner>,
    tools: ToolBox,
}

impl lume_app::AgentBackend for Critiqued {
    fn prepare(
        &self,
        _settings: &Settings,
        _run: RunId,
    ) -> Result<lume_app::ports::PreparedAgent, PortError> {
        Ok(lume_app::ports::PreparedAgent {
            runner: self.artist.clone(),
            tools: self.tools.clone(),
            model: "modelo".into(),
        })
    }

    fn prepare_critic(
        &self,
        _settings: &Settings,
        _run: RunId,
    ) -> Result<lume_app::ports::PreparedAgent, PortError> {
        Ok(lume_app::ports::PreparedAgent {
            runner: self.critic.clone(),
            tools: self.tools.clone(),
            model: "modelo".into(),
        })
    }
}

fn seta() -> EvalTask {
    EvalTask {
        id: "seta".into(),
        prompt: "una seta roja".into(),
        width: 8,
        height: 8,
        origin: String::new(),
    }
}

/// Las tres variantes hacen el mismo encargo, cada una en su sesión, y se
/// miden igual; el crítico pone la misma clase de nota a todas.
#[test]
fn each_variant_is_measured_the_same_way() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    // B: dibuja y promete en la primera vuelta; luego el crítico.
    let b = Arc::new(FakeProvider::new([
        reply(vec![
            text("Empiezo."),
            draw("t1", "fill_rect 1 1 6 6 #cc2222"),
        ]),
        reply(vec![text("Hecha. <promise>LISTO</promise>")]),
        verdict(),
    ]));
    // C: dos vueltas (la primera sin promesa) y, al retomar, promete.
    let c = Arc::new(FakeProvider::new([
        reply(vec![draw("t1", "fill_rect 1 1 6 6 #cc2222")]),
        reply(vec![text("Primera pasada.")]),
        reply(vec![text(
            "Retomo y la doy por buena. <promise>LISTO</promise>",
        )]),
        verdict(),
    ]));
    let critic = Arc::new(FakeProvider::new([verdict()]));
    let agents = Arc::new(Agents {
        providers: Mutex::new(vec![(Variant::B, b.clone()), (Variant::C, c.clone())]),
        critic,
    });
    let runner = EvalRunner {
        agents,
        store: Arc::new(MemoryStore::new()),
        settings: Settings {
            max_turns: 3,
            ..Settings::default()
        },
        critic: true,
    };
    let task = seta();
    let mut results = Vec::new();
    for (i, variant) in Variant::ALL.into_iter().enumerate() {
        let out = rt.block_on(runner.run(
            &task,
            variant,
            100 + i as u64,
            format!("k{i}"),
            CancellationToken::new(),
        ));
        results.push(out);
    }

    let a = &results[0].metrics;
    assert_eq!(a.variant, Variant::A);
    assert_eq!(a.key, "k0");
    assert_eq!(a.turns, 3, "las vueltas del plugin");
    assert_eq!(a.tokens, 3400);
    assert_eq!(a.cached_tokens, 9000);
    assert!(a.passed, "{a:?}");
    assert_eq!(
        a.end,
        "Terminado en la vuelta 3: el agente lo da por bueno."
    );
    assert_eq!(a.critic, Some(8.0));
    assert_eq!(a.issues, [0, 1, 1]);
    assert_eq!(a.approved, Some(true));
    assert!(a.error.is_none());
    assert_eq!(
        results[0].canvas.get(3, 3).unwrap(),
        Rgba::parse("#cc2222").unwrap()
    );

    let b_metrics = &results[1].metrics;
    assert_eq!(b_metrics.turns, 1);
    assert_eq!(b_metrics.tokens, 240);
    assert!(b_metrics.passed, "{b_metrics:?}");
    assert_eq!(b_metrics.critic, Some(8.0));
    assert!(b_metrics.seconds >= 0.0);

    let c_metrics = &results[2].metrics;
    assert_eq!(c_metrics.turns, 2, "{c_metrics:?}");
    assert!(c_metrics.passed);
    // C retoma la conversación de la vuelta 1.
    let requests = c.requests.lock().unwrap();
    assert!(requests[2].messages.len() > 1, "la vuelta 2 retoma");
    drop(requests);
    // B empieza siempre de cero (y dibuja sin crítico durante el bucle).
    assert_eq!(b.requests.lock().unwrap().len(), 3);
}

/// Si el agente no se puede preparar, queda como fallo en las métricas.
#[test]
fn a_missing_agent_is_a_failed_run() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let agents = Arc::new(Agents {
        providers: Mutex::new(Vec::new()),
        critic: Arc::new(FakeProvider::new([])),
    });
    let runner = EvalRunner {
        agents,
        store: Arc::new(MemoryStore::new()),
        settings: Settings::default(),
        critic: true,
    };
    let out = rt.block_on(runner.run(
        &seta(),
        Variant::B,
        1,
        "k1".into(),
        CancellationToken::new(),
    ));
    assert_eq!(
        out.metrics.error.as_deref(),
        Some("no disponible: sin agente")
    );
    assert!(!out.metrics.passed);
    assert_eq!(out.metrics.critic, None);
    assert!(!out.canvas.pixels().iter().any(|p| p.a > 0));
}

/// Las claves no se repiten y salen siempre iguales con la misma semilla.
#[test]
fn keys_are_unique_and_reproducible() {
    let keys = lume_app::eval::keys(42, 60);
    assert_eq!(keys.len(), 60);
    let mut sorted = keys.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), 60);
    assert_eq!(keys, lume_app::eval::keys(42, 60));
}
