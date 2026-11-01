//! `RefineLoop`: el bucle de refinado (el «ralph loop» del producto).
//!
//! Cada vuelta es una ejecución nueva del agente **con el contexto vacío**:
//! solo hereda el objetivo original y un bloque de estado (vuelta N de M,
//! notas del documento —`notas.md`—, comprobaciones que fallaron y lo que
//! dijo el crítico) más el lienzo actual como imagen. El agente actualiza las
//! notas con `write_notes`. Al terminar cada vuelta se guarda una versión del
//! documento con autor «agente» y el número de vuelta.
//!
//! El bucle termina cuando el agente escribe `<promise>LISTO</promise>` **y**
//! pasan las comprobaciones ([`CompletionPolicy`]), o al llegar a un límite:
//! vueltas, presupuesto de tokens, dos vueltas sin progreso o Detener. Si
//! termina sin éxito, entrega la última versión que pasó las comprobaciones
//! o, si no hay, la mejor según el crítico.
//!
//! Es un caso de uso de la aplicación: solo usa puertos (`AgentBackend`,
//! `DocumentStore`) y la sesión, así que lo puede lanzar la interfaz.

use std::sync::Arc;

use lume_domain::{Canvas, Origin, Rgba, RunId};
use tokio::sync::mpsc;

use crate::critic::{Critique, Verdict, critic_task, review};
use crate::ports::{
    AgentBackend, AgentEvent, AgentLimits, AgentTask, Attachment, CANVAS_ATTACHMENT,
    CancellationToken, ContentPart, Conversation, DocumentStore, EventSink, PortError, RunEnd,
    RunSummary, Usage,
};
use crate::session::SessionHandle;
use crate::settings::{ContextPolicy, Settings};

/// Bloque de estado de cada vuelta: `{turn}`, `{max}`, `{notes}`, `{extra}`.
pub const LOOP_STATE: &str = include_str!("../prompts/loop_state.md");
/// Comprobaciones que fallaron: `{items}`.
pub const LOOP_CHECKS: &str = include_str!("../prompts/loop_checks.md");
/// Problemas que vio el crítico: `{items}`.
pub const LOOP_CRITIC: &str = include_str!("../prompts/loop_critic.md");
/// Mensajes del usuario durante el bucle: `{items}`.
pub const LOOP_MESSAGES: &str = include_str!("../prompts/loop_messages.md");
/// Primera línea del bloque con contexto vacío (lo normal).
pub const LOOP_CONTEXT_FRESH: &str = include_str!("../prompts/loop_context_fresh.md");
/// Primera línea del bloque con la misma conversación (`Resume`).
pub const LOOP_CONTEXT_RESUME: &str = include_str!("../prompts/loop_context_resume.md");

/// Lo que escribe el agente cuando da el dibujo por terminado.
pub const PROMISE: &str = "<promise>LISTO</promise>";

/// Lado aproximado (en píxeles de pantalla) del lienzo que se adjunta.
const CANVAS_VIEW_SIDE: u32 = 512;

/// Presupuesto de tokens del bucle entero, por defecto.
pub const DEFAULT_TOKEN_BUDGET: u64 = 1_500_000;

/// Lo que va pasando en el bucle (para la interfaz).
#[derive(Debug, Clone, PartialEq)]
pub enum LoopEvent {
    TurnStarted {
        turn: u32,
        of: u32,
    },
    /// Algo que hizo el agente en la vuelta en curso.
    Agent(AgentEvent),
    /// El veredicto del crítico de la vuelta.
    Critic {
        turn: u32,
        verdict: Verdict,
    },
    /// Terminó una vuelta: su versión y sus comprobaciones.
    TurnFinished {
        turn: u32,
        version: u32,
        summary: RunSummary,
        checks: Vec<CheckResult>,
        promised: bool,
    },
}

/// Para mandar mensajes al bucle mientras dibuja: entran en el bloque de
/// estado de la vuelta siguiente, sin detenerlo.
#[derive(Debug, Clone)]
pub struct LoopMessages(mpsc::UnboundedSender<String>);

impl LoopMessages {
    pub fn send(&self, text: impl Into<String>) {
        let text = text.into();
        if !text.trim().is_empty() {
            let _ = self.0.send(text);
        }
    }
}

/// Canal de eventos del bucle.
#[derive(Debug, Clone)]
pub struct LoopSink(mpsc::UnboundedSender<LoopEvent>);

impl LoopSink {
    pub fn channel() -> (LoopSink, mpsc::UnboundedReceiver<LoopEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();
        (LoopSink(tx), rx)
    }

    pub fn emit(&self, event: LoopEvent) {
        let _ = self.0.send(event);
    }
}

// ---------------------------------------------------------------------------
// Comprobaciones
// ---------------------------------------------------------------------------

/// El resultado de una comprobación.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub passed: bool,
    /// Qué se encontró (lo que lee el agente si falla).
    pub detail: String,
}

/// Lo que se sabe de una vuelta al terminar.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TurnReport {
    /// El agente escribió la promesa.
    pub promised: bool,
    pub canvas_empty: bool,
    /// Problemas de la vuelta (el último lote falló, se cortó por un límite).
    pub errors: Vec<String>,
    /// Hay paleta fija.
    pub fixed_palette: bool,
    /// Colores del lienzo fuera de la paleta fija.
    pub outside_palette: Vec<Rgba>,
    /// Veredicto del crítico, si está activo.
    pub verdict: Option<Verdict>,
}

/// Cuándo está terminado un dibujo (patrón Specification): la promesa y que
/// pasen todas las comprobaciones que aplican.
#[derive(Debug, Clone, Copy, Default)]
pub struct CompletionPolicy;

impl CompletionPolicy {
    /// Las comprobaciones objetivas de una vuelta (la promesa va aparte).
    pub fn checks(report: &TurnReport) -> Vec<CheckResult> {
        let mut out = vec![
            CheckResult {
                name: "lienzo",
                passed: !report.canvas_empty,
                detail: if report.canvas_empty {
                    "El lienzo está vacío.".into()
                } else {
                    "El lienzo tiene dibujo.".into()
                },
            },
            CheckResult {
                name: "errores",
                passed: report.errors.is_empty(),
                detail: if report.errors.is_empty() {
                    "La vuelta terminó sin errores.".into()
                } else {
                    report.errors.join(" ")
                },
            },
        ];
        if report.fixed_palette {
            let colors: Vec<String> = report.outside_palette.iter().map(hex).collect();
            out.push(CheckResult {
                name: "paleta",
                passed: colors.is_empty(),
                detail: if colors.is_empty() {
                    "Todos los colores son de la paleta fija.".into()
                } else {
                    format!(
                        "Hay colores fuera de la paleta fija: {}.",
                        colors.join(", ")
                    )
                },
            });
        }
        if let Some(verdict) = &report.verdict {
            let (passed, detail) = match verdict {
                Verdict::Ready(critique) => match critique.serious().len() {
                    0 => (true, "El crítico no ve problemas graves.".to_string()),
                    1 => (false, "El crítico ve 1 problema grave.".to_string()),
                    n => (false, format!("El crítico ve {n} problemas graves.")),
                },
                // Sin veredicto no se bloquea el bucle.
                Verdict::None { reason } => {
                    (true, format!("Sin veredicto del crítico ({reason})."))
                }
            };
            out.push(CheckResult {
                name: "crítico",
                passed,
                detail,
            });
        }
        out
    }

    /// ¿Está terminado? La promesa sola no basta.
    pub fn is_done(report: &TurnReport) -> bool {
        report.promised && Self::checks(report).iter().all(|c| c.passed)
    }
}

fn hex(c: &Rgba) -> String {
    if c.a == 255 {
        format!("#{:02x}{:02x}{:02x}", c.r, c.g, c.b)
    } else {
        c.to_string()
    }
}

// ---------------------------------------------------------------------------
// Resultado
// ---------------------------------------------------------------------------

/// Por qué terminó el bucle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoopEnd {
    /// Promesa y comprobaciones: terminado.
    Done,
    /// Se hicieron todas las vueltas.
    MaxTurns,
    /// Se gastó el presupuesto de tokens.
    TokenBudget,
    /// Dos vueltas seguidas sin avanzar.
    NoProgress,
    /// Detener.
    Cancelled,
    /// Falló una vuelta (el proveedor, la sesión…).
    Failed(String),
}

/// Qué versión se entrega y por qué.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    /// La última vuelta (terminada, o sin nada mejor).
    Last,
    /// La última que pasó las comprobaciones.
    LastPassing,
    /// La de menos problemas graves según el crítico.
    BestByCritic,
}

/// Resumen del bucle.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopSummary {
    pub turns: u32,
    /// Versiones guardadas, una por vuelta.
    pub versions: Vec<u32>,
    pub usage: Usage,
    /// Último texto del agente.
    pub final_text: String,
    pub end: LoopEnd,
    /// Comprobaciones de la última vuelta.
    pub checks: Vec<CheckResult>,
    /// Versión entregada (la que queda en el lienzo) y por qué.
    pub delivered: Option<(u32, Delivery)>,
    /// Presupuesto de tokens del bucle.
    pub token_budget: u64,
}

impl LoopSummary {
    /// Cómo terminó, en una frase para la interfaz.
    pub fn reason(&self) -> String {
        let mut text = match &self.end {
            LoopEnd::Done => format!(
                "Terminado en la vuelta {}: el agente lo da por bueno y pasan todas las comprobaciones.",
                self.turns
            ),
            LoopEnd::MaxTurns if self.turns == 1 => "Se hizo la única vuelta sin terminar.".into(),
            LoopEnd::MaxTurns => format!("Se hicieron las {} vueltas sin terminar.", self.turns),
            LoopEnd::TokenBudget => format!(
                "Se paró al gastar el presupuesto de {} tokens (usó {}).",
                self.token_budget,
                self.usage.total()
            ),
            LoopEnd::NoProgress => "Se paró porque dos vueltas seguidas no avanzaron.".into(),
            LoopEnd::Cancelled => format!("Detenido en la vuelta {}.", self.turns.max(1)),
            LoopEnd::Failed(e) => format!("Falló la vuelta {}: {e}", self.turns + 1),
        };
        if self.end != LoopEnd::Done
            && let Some((version, how)) = self.delivered
        {
            let why = match how {
                Delivery::Last => "la última",
                Delivery::LastPassing => "la última que pasó las comprobaciones",
                Delivery::BestByCritic => "la mejor según el crítico",
            };
            text.push_str(&format!(" Se entrega la versión {version} ({why})."));
        }
        text
    }
}

/// Lo que se recuerda de cada vuelta para elegir qué entregar.
#[derive(Debug, Clone)]
struct TurnRecord {
    version: u32,
    passed: bool,
    serious: Option<usize>,
}

// ---------------------------------------------------------------------------
// El bucle
// ---------------------------------------------------------------------------

/// El bucle de refinado de un encargo.
pub struct RefineLoop {
    pub backend: Arc<dyn AgentBackend>,
    pub session: SessionHandle,
    pub store: Arc<dyn DocumentStore>,
    /// Ajustes del momento (proveedor, vueltas, sin progreso, crítico…).
    pub settings: Settings,
    /// Límites de cada vuelta.
    pub turn_limits: AgentLimits,
    /// Tokens del bucle entero.
    pub token_budget: u64,
    /// Trabajo de animación: cada vuelta recibe `anim.js` y la base.
    pub animation: bool,
    /// Trabajo vectorial: cada vuelta recibe `dibujo.js`.
    pub vector: bool,
    /// Mensajes del usuario pendientes (ver [`RefineLoop::messages`]).
    inbox: std::sync::Mutex<mpsc::UnboundedReceiver<String>>,
    outbox: mpsc::UnboundedSender<String>,
}

/// Lo que deja una vuelta.
struct TurnOutcome {
    summary: RunSummary,
    task: AgentTask,
    /// El último lote de comandos falló.
    last_batch_failed: bool,
    /// Celdas que cambiaron en la vuelta.
    changed: usize,
    canvas: Canvas,
}

impl RefineLoop {
    pub fn new(
        backend: Arc<dyn AgentBackend>,
        session: SessionHandle,
        store: Arc<dyn DocumentStore>,
        settings: Settings,
    ) -> Self {
        let (outbox, inbox) = mpsc::unbounded_channel();
        Self {
            backend,
            session,
            store,
            settings,
            turn_limits: AgentLimits::default(),
            token_budget: DEFAULT_TOKEN_BUDGET,
            animation: false,
            vector: false,
            inbox: std::sync::Mutex::new(inbox),
            outbox,
        }
    }

    /// Para mandar mensajes al bucle mientras dibuja.
    pub fn messages(&self) -> LoopMessages {
        LoopMessages(self.outbox.clone())
    }

    /// Los mensajes que llegaron desde la última vuelta.
    fn take_messages(&self) -> Vec<String> {
        let mut inbox = self.inbox.lock().unwrap_or_else(|e| e.into_inner());
        let mut out = Vec::new();
        while let Ok(text) = inbox.try_recv() {
            out.push(text.trim().to_string());
        }
        out
    }

    /// Ejecuta el bucle para `goal` (con sus imágenes de referencia). Todo
    /// lo que haga el agente queda como `Origin::Agent { run }`.
    pub async fn run(
        &self,
        run: RunId,
        goal: &str,
        references: Vec<Attachment>,
        events: LoopSink,
        cancel: CancellationToken,
    ) -> LoopSummary {
        let max = self.settings.max_turns.max(1);
        let mut summary = LoopSummary {
            turns: 0,
            versions: Vec::new(),
            usage: Usage::default(),
            final_text: String::new(),
            end: LoopEnd::MaxTurns,
            checks: Vec::new(),
            delivered: None,
            token_budget: self.token_budget,
        };
        let mut records: Vec<TurnRecord> = Vec::new();
        let mut extra = String::new();
        let mut stalled = 0u32;
        let mut previous: Option<Conversation> = None;
        for turn in 1..=max {
            if cancel.is_cancelled() {
                summary.end = LoopEnd::Cancelled;
                break;
            }
            events.emit(LoopEvent::TurnStarted { turn, of: max });
            let remaining = self.token_budget.saturating_sub(summary.usage.total());
            let turn_extra = format!("{extra}{}", messages_section(&self.take_messages()));
            let resume = match self.settings.context {
                ContextPolicy::Resume => previous.take(),
                ContextPolicy::Fresh => None,
            };
            let outcome = match self
                .turn(
                    run,
                    turn,
                    max,
                    goal,
                    &references,
                    &turn_extra,
                    resume,
                    remaining,
                    &events,
                    &cancel,
                )
                .await
            {
                Ok(outcome) => outcome,
                Err(err) => {
                    summary.end = LoopEnd::Failed(err.to_string());
                    break;
                }
            };
            summary.turns = turn;
            summary.usage.add(outcome.summary.usage);
            previous = outcome.summary.conversation.clone();
            if !outcome.summary.final_text.trim().is_empty() {
                summary.final_text = outcome.summary.final_text.clone();
            }
            let cancelled = outcome.summary.end == RunEnd::Cancelled || cancel.is_cancelled();

            // El crítico mira el resultado de la vuelta.
            let verdict = if self.settings.critic && !cancelled {
                let verdict = self.critique(run, &outcome.task, &cancel).await;
                events.emit(LoopEvent::Critic {
                    turn,
                    verdict: verdict.clone(),
                });
                Some(verdict)
            } else {
                None
            };

            let report = self.report(&outcome, verdict.clone()).await;
            let checks = CompletionPolicy::checks(&report);
            let passed = checks.iter().all(|c| c.passed);
            let version = match self
                .session
                .create_version(Origin::Agent { run }, format!("Vuelta {turn} de {max}"))
                .await
            {
                Ok(version) => version,
                Err(err) => {
                    summary.end = LoopEnd::Failed(err.to_string());
                    break;
                }
            };
            summary.versions.push(version);
            records.push(TurnRecord {
                version,
                passed,
                serious: match &verdict {
                    Some(Verdict::Ready(c)) => Some(c.serious().len()),
                    _ => None,
                },
            });
            events.emit(LoopEvent::TurnFinished {
                turn,
                version,
                summary: outcome.summary.clone(),
                checks: checks.clone(),
                promised: report.promised,
            });
            summary.checks = checks.clone();

            if cancelled {
                summary.end = LoopEnd::Cancelled;
                break;
            }
            if CompletionPolicy::is_done(&report) {
                summary.end = LoopEnd::Done;
                break;
            }
            if summary.usage.total() >= self.token_budget {
                summary.end = LoopEnd::TokenBudget;
                break;
            }
            stalled = if outcome.changed < progress_threshold(&outcome.canvas) {
                stalled + 1
            } else {
                0
            };
            if self.settings.stop_without_progress && stalled >= 2 {
                summary.end = LoopEnd::NoProgress;
                break;
            }
            extra = next_turn_notes(&checks, verdict.as_ref());
        }
        summary.delivered = self.deliver(run, &summary.end, &records).await;
        summary
    }

    /// Una vuelta: agente nuevo, encargo con el bloque de estado y el lienzo.
    #[allow(clippy::too_many_arguments)]
    async fn turn(
        &self,
        run: RunId,
        turn: u32,
        max: u32,
        goal: &str,
        references: &[Attachment],
        extra: &str,
        resume: Option<Conversation>,
        remaining_tokens: u64,
        events: &LoopSink,
        cancel: &CancellationToken,
    ) -> Result<TurnOutcome, PortError> {
        let document = self.snapshot().await?;
        let before = document.canvas().clone();
        let has_drawing = before.pixels().iter().any(|p| p.a > 0);
        let mut attachments = references.to_vec();
        // En un trabajo vectorial, el dibujo es dibujo.js: el lienzo de
        // píxeles no se adjunta (el agente vería otra cosa).
        if has_drawing && !self.vector {
            let side = before.width().max(before.height()).max(1);
            let scale = (CANVAS_VIEW_SIDE / side).clamp(1, 32);
            let png = self
                .store
                .encode_png(&before, scale)
                .map_err(|e| PortError::Invalid(e.message))?;
            attachments.push(Attachment {
                label: CANVAS_ATTACHMENT.to_string(),
                media_type: "image/png".into(),
                data: png,
            });
        }
        let palette = document
            .palette
            .clone()
            .filter(|p| p.name.is_some() && !self.vector);
        // Trabajo de animación: el agente recibe anim.js y la base.
        let anim = match (&document.animation, self.animation) {
            (Some(animation), true) => {
                let base = animation
                    .base
                    .clone()
                    .unwrap_or_else(|| lume_domain::anim::Image::from_canvas(document.canvas()));
                let base_png = self
                    .store
                    .encode_image(&base)
                    .map_err(|e| PortError::Invalid(e.message))?;
                Some(crate::ports::AnimTask {
                    source: animation.source.clone(),
                    base_png,
                })
            }
            _ => None,
        };
        let prepared = self.backend.prepare(&self.settings, run)?;
        let mut limits = self.turn_limits;
        limits.max_tokens = limits.max_tokens.min(remaining_tokens.max(1));
        let task = AgentTask {
            run,
            goal: goal.to_string(),
            context: state_block(
                turn,
                max,
                &document.notes,
                extra,
                self.settings.context == ContextPolicy::Resume,
            ),
            attachments,
            model: prepared.model.clone(),
            limits,
            canvas: (before.width(), before.height()),
            palette,
            continuing: has_drawing,
            resume,
            anim,
            vector: match (&document.vector, self.vector) {
                (Some(drawing), true) => Some(crate::ports::VecTask {
                    source: drawing.source.clone(),
                }),
                _ => None,
            },
        };

        // Los eventos del agente salen como eventos del bucle; de paso se
        // mira cómo acabó el último lote de comandos.
        let (sink, mut agent_events) = EventSink::channel();
        let running = prepared
            .runner
            .run(task.clone(), prepared.tools, sink, cancel.clone());
        tokio::pin!(running);
        let mut last_batch_failed = false;
        let result = loop {
            tokio::select! {
                event = agent_events.recv() => {
                    if let Some(event) = event {
                        if let Some(failed) = batch_failed(&event) {
                            last_batch_failed = failed;
                        }
                        events.emit(LoopEvent::Agent(event));
                    }
                }
                result = &mut running => break result,
            }
        };
        while let Ok(event) = agent_events.try_recv() {
            if let Some(failed) = batch_failed(&event) {
                last_batch_failed = failed;
            }
            events.emit(LoopEvent::Agent(event));
        }
        let summary = result?;
        let after = self.snapshot().await?.canvas().clone();
        Ok(TurnOutcome {
            summary,
            task,
            last_batch_failed,
            changed: lume_domain::canvas::diff(&before, &after).len(),
            canvas: after,
        })
    }

    async fn snapshot(&self) -> Result<lume_domain::Document, PortError> {
        self.session
            .snapshot()
            .await
            .map_err(|e| PortError::Unavailable(e.to_string()))
    }

    /// Lanza el crítico (nunca falla: sin él, `Verdict::None`).
    async fn critique(&self, run: RunId, task: &AgentTask, cancel: &CancellationToken) -> Verdict {
        match self.backend.prepare_critic(&self.settings, run) {
            Ok(critic) => {
                review(
                    critic,
                    critic_task(task),
                    EventSink::channel().0,
                    cancel.clone(),
                )
                .await
            }
            Err(err) => Verdict::None {
                reason: format!("no se pudo preparar el crítico: {err}"),
            },
        }
    }

    async fn report(&self, outcome: &TurnOutcome, verdict: Option<Verdict>) -> TurnReport {
        let canvas = &outcome.canvas;
        let mut errors = Vec::new();
        if outcome.last_batch_failed {
            errors.push("El último lote de comandos tuvo errores.".to_string());
        }
        match outcome.summary.end {
            RunEnd::StepLimit => errors.push("La vuelta se cortó por el límite de pasos.".into()),
            RunEnd::TokenLimit => errors.push("La vuelta se cortó por el límite de tokens.".into()),
            RunEnd::Finished | RunEnd::Cancelled => {}
        }
        let document = self.snapshot().await.ok();
        let palette = document
            .as_ref()
            .and_then(|d| d.palette.clone())
            .filter(|p| p.name.is_some());
        // En un trabajo vectorial, el dibujo es su dibujo.js (el lienzo de
        // píxeles no se toca).
        let canvas_empty = if self.vector {
            document
                .as_ref()
                .and_then(|d| d.vector.as_ref())
                .is_none_or(|v| v.source.trim().is_empty())
        } else {
            !canvas.pixels().iter().any(|p| p.a > 0)
        };
        TurnReport {
            promised: promised(&outcome.summary.final_text),
            canvas_empty,
            errors,
            fixed_palette: palette.is_some(),
            outside_palette: palette
                .map(|p| p.colors_outside(canvas))
                .unwrap_or_default(),
            verdict,
        }
    }

    /// Si no terminó con éxito, deja en el lienzo la mejor versión.
    async fn deliver(
        &self,
        run: RunId,
        end: &LoopEnd,
        records: &[TurnRecord],
    ) -> Option<(u32, Delivery)> {
        let last = records.last()?;
        if *end == LoopEnd::Done {
            return Some((last.version, Delivery::Last));
        }
        let choice = if let Some(r) = records.iter().rev().find(|r| r.passed) {
            (r.version, Delivery::LastPassing)
        } else if let Some((_, version)) = records
            .iter()
            .filter_map(|r| r.serious.map(|s| (s, r.version)))
            // Menos problemas; a igualdad, la más reciente.
            .min_by_key(|(s, v)| (*s, std::cmp::Reverse(*v)))
        {
            (version, Delivery::BestByCritic)
        } else {
            (last.version, Delivery::Last)
        };
        if choice.0 != last.version {
            let _ = self
                .session
                .restore_version(choice.0, Origin::Agent { run })
                .await;
        }
        Some(choice)
    }
}

/// ¿Cierra el agente su último mensaje con la promesa? Citarla en medio del
/// texto («escribiré <promise>LISTO</promise> cuando…») no cuenta.
pub fn promised(final_text: &str) -> bool {
    final_text.trim_end().ends_with(PROMISE)
}

/// Celdas que tiene que cambiar una vuelta para contar como progreso (el
/// 0,5 % del lienzo, al menos una).
pub fn progress_threshold(canvas: &Canvas) -> usize {
    ((canvas.width() as usize * canvas.height() as usize) / 200).max(1)
}

/// ¿Es el resultado de un lote de comandos? Si lo es, ¿falló algo?
fn batch_failed(event: &AgentEvent) -> Option<bool> {
    let AgentEvent::ToolResult { name, output } = event else {
        return None;
    };
    let text: String = output
        .content
        .iter()
        .filter_map(|p| match p {
            ContentPart::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    if name != "draw" && !text.contains("batch: ") {
        return None;
    }
    let failed = text
        .lines()
        .filter_map(|l| l.trim().strip_prefix("batch: "))
        .filter_map(|rest| {
            rest.split(|c: char| !c.is_ascii_digit())
                .filter(|s| !s.is_empty())
                .nth(1)
                .and_then(|n| n.parse::<usize>().ok())
        })
        .sum::<usize>();
    Some(output.is_error || failed > 0)
}

/// Lo que la siguiente vuelta tiene que saber de esta: comprobaciones que
/// fallaron y problemas del crítico.
fn next_turn_notes(checks: &[CheckResult], verdict: Option<&Verdict>) -> String {
    let mut out = String::new();
    let failed: Vec<String> = checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| format!("- {}", c.detail))
        .collect();
    if !failed.is_empty() {
        out.push('\n');
        out.push_str(fill(LOOP_CHECKS, &[("items", &failed.join("\n"))]).trim_end());
        out.push('\n');
    }
    if let Some(Verdict::Ready(critique)) = verdict {
        let issues: Vec<&crate::critic::Issue> = critique.issues.iter().collect();
        if !issues.is_empty() {
            let items = Critique::issues_text(&issues);
            out.push('\n');
            out.push_str(fill(LOOP_CRITIC, &[("items", items.trim_end())]).trim_end());
            out.push('\n');
        }
    }
    out
}

/// Los mensajes del usuario para el bloque de estado (vacío si no hay).
fn messages_section(messages: &[String]) -> String {
    if messages.is_empty() {
        return String::new();
    }
    let items: Vec<String> = messages.iter().map(|m| format!("- {m}")).collect();
    format!(
        "\n{}\n",
        fill(LOOP_MESSAGES, &[("items", &items.join("\n"))]).trim_end()
    )
}

/// Rellena los huecos `{nombre}` en una sola pasada: lo que se mete no se
/// vuelve a interpretar.
pub(crate) fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let value = after.find('}').and_then(|end| {
            let key = &after[..end];
            values
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v, end))
        });
        match value {
            Some((value, end)) => {
                out.push_str(value);
                rest = &after[end + 1..];
            }
            None => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// El bloque de estado de una vuelta. `extra`: comprobaciones que fallaron,
/// lo que dijo el crítico y los mensajes del usuario. `resume`: la vuelta
/// sigue la conversación anterior.
pub fn state_block(turn: u32, max: u32, notes: &str, extra: &str, resume: bool) -> String {
    let context = if resume {
        LOOP_CONTEXT_RESUME
    } else {
        LOOP_CONTEXT_FRESH
    };
    let notes = if notes.trim().is_empty() {
        "(todavía no hay notas)"
    } else {
        notes.trim()
    };
    fill(
        LOOP_STATE,
        &[
            ("turn", &turn.to_string()),
            ("max", &max.to_string()),
            ("context", context.trim()),
            ("notes", notes),
            ("extra", extra),
        ],
    )
    .trim_end()
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_state_block_is_filled() {
        let block = state_block(2, 8, "Hice el líquido; falta el tapón.", "", false);
        assert!(
            block.starts_with("## Bucle de refinado · vuelta 2 de 8"),
            "{block}"
        );
        assert!(block.contains("Hice el líquido; falta el tapón."));
        assert!(block.contains(PROMISE));
        assert!(block.contains("empieza con el contexto vacío"));
        let resumed = state_block(2, 8, "", "", true);
        assert!(
            resumed.contains("Sigues la misma conversación"),
            "{resumed}"
        );
        for hole in ["{turn}", "{max}", "{notes}", "{extra}", "{context}"] {
            assert!(!block.contains(hole), "{hole} sin rellenar");
        }
        assert!(state_block(1, 3, "  ", "", false).contains("(todavía no hay notas)"));
        // Unas notas con «{max}» se quedan como están.
        assert!(state_block(1, 3, "usa {max}", "", false).contains("usa {max}"));
    }

    /// Encontrado probando con Claude Code: citar la promesa en las notas no
    /// es prometer.
    #[test]
    fn only_a_closing_promise_counts() {
        assert!(promised("Listo.\n<promise>LISTO</promise>"));
        assert!(promised("Hecho. <promise>LISTO</promise>  \n"));
        assert!(!promised(
            "Falta la vuelta 2.\n- Después, escribir la promesa <promise>LISTO</promise> (solo cuando estén los dos)."
        ));
        assert!(!promised("Todavía no."));
    }

    #[test]
    fn checks_follow_the_report() {
        let report = TurnReport {
            promised: true,
            canvas_empty: true,
            ..TurnReport::default()
        };
        let checks = CompletionPolicy::checks(&report);
        assert_eq!(checks.len(), 2, "sin paleta fija ni crítico");
        assert!(!checks[0].passed);
        assert!(
            !CompletionPolicy::is_done(&report),
            "la promesa sola no basta"
        );

        let report = TurnReport {
            promised: true,
            fixed_palette: true,
            outside_palette: vec![Rgba::rgb(255, 0, 0)],
            verdict: Some(Verdict::None { reason: "x".into() }),
            ..TurnReport::default()
        };
        let checks = CompletionPolicy::checks(&report);
        assert_eq!(checks.len(), 4);
        assert_eq!(
            checks[2].detail,
            "Hay colores fuera de la paleta fija: #ff0000."
        );
        assert!(checks[3].passed, "sin veredicto no bloquea");
        assert!(!CompletionPolicy::is_done(&report));
    }

    #[test]
    fn batch_results_are_recognised() {
        let ev = |name: &str, t: &str| AgentEvent::ToolResult {
            name: name.into(),
            output: crate::ports::ToolOutput::text(t),
        };
        assert_eq!(
            batch_failed(&ev("draw", "batch: 3 ok, 0 failed")),
            Some(false)
        );
        assert_eq!(
            batch_failed(&ev("draw", "line 2: x\nbatch: 1 ok, 1 failed")),
            Some(true)
        );
        assert_eq!(
            batch_failed(&ev("Bash", "batch: 2 ok, 0 failed\nbatch: 1 ok, 2 failed")),
            Some(true)
        );
        assert_eq!(batch_failed(&ev("Read", "imagen")), None);
        assert_eq!(batch_failed(&ev("view_canvas", "Lienzo")), None);
    }
}
