//! `lumeart eval` en la aplicación: hace un encargo con una variante del
//! bucle sobre una sesión nueva y mide lo que pasó.
//!
//! - **A** (plugin oficial): una sola ejecución del agente; su runner lleva
//!   el bucle (el Stop hook del plugin le devuelve el encargo) y dice cuántas
//!   vueltas dio en [`RunSummary::loops`](crate::ports::RunSummary).
//! - **B** y **C**: el [`RefineLoop`] con el contexto vacío o retomando la
//!   conversación.
//!
//! Las tres dibujan sin crítico (A no puede tenerlo); al terminar, el mismo
//! crítico mira cada resultado y le pone nota.

use std::sync::Arc;
use std::time::Instant;

use lume_domain::eval::{EvalTask, RunMetrics, Variant, count_issues, critic_score};
use lume_domain::{Canvas, Document, RunId};

use crate::art::ArtAgent;
use crate::critic::{Verdict, critic_task, review};
use crate::ports::{
    AgentLimits, AgentTask, BoxFuture, CancellationToken, DocumentStore, EventSink, PortError,
    RunEnd,
};
use crate::refine::{LoopEnd, LoopSink, RefineLoop, promised};
use crate::session::{Session, SessionHandle};
use crate::settings::{ContextPolicy, Settings};

/// Monta el agente de una variante sobre la sesión de un encargo (A: el
/// plugin oficial; B y C: el agente de siempre). Lo implementa la raíz.
pub trait EvalAgents: Send + Sync {
    fn agent(
        &self,
        session: SessionHandle,
        variant: Variant,
        run: RunId,
    ) -> BoxFuture<'_, Result<ArtAgent, PortError>>;
}

/// Lo que dio un encargo: las métricas y el dibujo final.
#[derive(Debug, Clone)]
pub struct EvalOutcome {
    pub metrics: RunMetrics,
    pub canvas: Canvas,
}

/// Hace encargos con las variantes.
pub struct EvalRunner {
    pub agents: Arc<dyn EvalAgents>,
    pub store: Arc<dyn DocumentStore>,
    /// Proveedor, modelo y vueltas (las mismas para las tres variantes).
    pub settings: Settings,
    /// Pasar el crítico por cada resultado.
    pub critic: bool,
}

impl EvalRunner {
    /// Los ajustes con que dibuja una variante: sin crítico y, en C,
    /// retomando la conversación.
    pub fn settings_for(&self, variant: Variant) -> Settings {
        Settings {
            context: if variant == Variant::C {
                ContextPolicy::Resume
            } else {
                ContextPolicy::Fresh
            },
            critic: false,
            ..self.settings.clone()
        }
    }

    /// Hace `task` con `variant` en una sesión nueva. Nunca falla: lo que
    /// salga mal queda en las métricas (`error`).
    pub async fn run(
        &self,
        task: &EvalTask,
        variant: Variant,
        run: RunId,
        key: String,
        cancel: CancellationToken,
    ) -> EvalOutcome {
        let mut metrics = RunMetrics {
            task: task.id.clone(),
            variant,
            key,
            critic: None,
            approved: None,
            issues: [0; 3],
            passed: false,
            turns: 0,
            tokens: 0,
            cached_tokens: 0,
            seconds: 0.0,
            end: String::new(),
            error: None,
        };
        let document = match Document::new(&task.id, i64::from(task.width), i64::from(task.height))
        {
            Ok(document) => document,
            Err(err) => {
                metrics.error = Some(err.message);
                return EvalOutcome {
                    metrics,
                    canvas: Canvas::new(1, 1).expect("1x1 siempre vale"),
                };
            }
        };
        let blank = document.canvas().clone();
        let session = Session::spawn(document, self.store.clone());
        let handle = session.handle.clone();
        let settings = self.settings_for(variant);
        let agent = match self.agents.agent(handle.clone(), variant, run).await {
            Ok(agent) => agent,
            Err(err) => {
                metrics.error = Some(err.to_string());
                metrics.end = format!("No se pudo preparar el agente: {err}");
                session.close().await;
                return EvalOutcome {
                    metrics,
                    canvas: blank,
                };
            }
        };

        let started = Instant::now();
        let artist_task = AgentTask {
            run,
            goal: task.prompt.clone(),
            context: String::new(),
            attachments: Vec::new(),
            model: String::new(),
            limits: AgentLimits::default(),
            canvas: (task.width, task.height),
            palette: None,
            continuing: false,
            resume: None,
            anim: None,
            vector: None,
        };
        let failure = match variant {
            Variant::A => self
                .run_plugin(
                    &agent,
                    &settings,
                    artist_task.clone(),
                    &mut metrics,
                    &cancel,
                )
                .await
                .err(),
            Variant::B | Variant::C => {
                let refine = RefineLoop::new(
                    agent.backend.clone(),
                    handle.clone(),
                    self.store.clone(),
                    settings.clone(),
                );
                let (sink, _events) = LoopSink::channel();
                let summary = refine
                    .run(run, &task.prompt, Vec::new(), sink, cancel.clone())
                    .await;
                metrics.turns = summary.turns;
                metrics.tokens = summary.usage.total();
                metrics.cached_tokens = summary.usage.cached_tokens;
                metrics.passed = summary.end == LoopEnd::Done;
                metrics.end = summary.reason();
                match summary.end {
                    LoopEnd::Failed(message) => Some(message),
                    _ => None,
                }
            }
        };
        metrics.seconds = started.elapsed().as_secs_f64();
        let canvas = handle.view().borrow().canvas.as_ref().clone();
        let empty = !canvas.pixels().iter().any(|p| p.a > 0);
        if let Some(message) = failure
            && empty
        {
            metrics.error = Some(message);
        }

        if self.critic && !empty && !cancel.is_cancelled() {
            self.critique(&agent, &settings, &artist_task, &mut metrics, &cancel)
                .await;
        }
        drop(agent);
        session.close().await;
        EvalOutcome { metrics, canvas }
    }

    /// Variante A: una sola ejecución; el runner lleva el bucle.
    async fn run_plugin(
        &self,
        agent: &ArtAgent,
        settings: &Settings,
        mut task: AgentTask,
        metrics: &mut RunMetrics,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        let prepared = agent
            .backend
            .prepare(settings, task.run)
            .map_err(|e| e.to_string())?;
        task.model = prepared.model.clone();
        let (sink, _events) = EventSink::channel();
        let result = prepared
            .runner
            .run(task, prepared.tools, sink, cancel.clone())
            .await;
        let summary = match result {
            Ok(summary) => summary,
            Err(err) => {
                metrics.end = format!("Falló: {err}");
                return Err(err.to_string());
            }
        };
        metrics.turns = summary.loops.unwrap_or(1);
        metrics.tokens = summary.usage.total();
        metrics.cached_tokens = summary.usage.cached_tokens;
        let promise = promised(&summary.final_text);
        metrics.passed = promise && summary.end == RunEnd::Finished;
        metrics.end = match (&summary.end, promise) {
            (RunEnd::Finished, true) => format!(
                "Terminado en la vuelta {}: el agente lo da por bueno.",
                metrics.turns
            ),
            (RunEnd::Finished, false) => {
                format!("Se hicieron {} vueltas sin la promesa.", metrics.turns)
            }
            (RunEnd::StepLimit, _) => "Se cortó por el límite de pasos.".into(),
            (RunEnd::TokenLimit, _) => "Se cortó por el límite de tokens.".into(),
            (RunEnd::Cancelled, _) => "Detenido.".into(),
        };
        Ok(())
    }

    /// El crítico mira el resultado y le pone nota.
    async fn critique(
        &self,
        agent: &ArtAgent,
        settings: &Settings,
        artist_task: &AgentTask,
        metrics: &mut RunMetrics,
        cancel: &CancellationToken,
    ) {
        let critic = match agent.backend.prepare_critic(settings, artist_task.run) {
            Ok(critic) => critic,
            Err(_) => return,
        };
        let mut task = critic_task(artist_task);
        task.model = critic.model.clone();
        let (sink, _events) = EventSink::channel();
        if let Verdict::Ready(c) = review(critic, task, sink, cancel.clone()).await {
            let issues = count_issues(c.issues.iter().map(|i| i.severity.as_str()));
            metrics.issues = issues;
            metrics.critic = Some(critic_score(issues));
            metrics.approved = Some(c.approved);
        }
    }
}

/// Las claves neutras de una evaluación: una por ejecución, sin repetir.
pub fn keys(seed: u64, count: usize) -> Vec<String> {
    let mut rng = lume_domain::eval::Rng::new(seed);
    let mut out: Vec<String> = Vec::with_capacity(count);
    while out.len() < count {
        let key = rng.key();
        if !out.contains(&key) {
            out.push(key);
        }
    }
    out
}
