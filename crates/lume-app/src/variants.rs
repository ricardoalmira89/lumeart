//! Variantes de una animación: N ejecuciones del agente en paralelo, cada una
//! sobre su propia copia del documento, que cambian `anim.js` de formas
//! distintas a partir de la misma petición. Elegir una la convierte en la
//! animación del documento (lo hace quien llama: `AnimEdit::Source` y una
//! versión nueva). «Mezclar» lanza una variante más que combina las demás.

use std::sync::{Arc, Mutex};

use lume_domain::anim::Image;
use lume_domain::{Document, RunId};

use crate::art::{ArtAgents, ArtNotify};
use crate::ports::{
    AgentLimits, AgentTask, AnimTask, CancellationToken, DocumentStore, EventSink, PortError,
};
use crate::session::Session;
use crate::settings::SettingsStore;

/// Plantilla del encargo de cada variante: `{letter}`, `{count}`,
/// `{prompt}` y `{focus}`.
pub const VARIANT_GOAL: &str = include_str!("../prompts/variant_goal.md");

/// Plantilla de un encargo de animación desde el panel: `{prompt}`.
pub const ANIM_GOAL: &str = include_str!("../prompts/anim_goal.md");

/// La petición del usuario como trabajo de animación (cambiar `anim.js`).
pub fn anim_goal(prompt: &str) -> String {
    crate::refine::fill(ANIM_GOAL, &[("prompt", prompt.trim())])
}

/// Primer número de ejecución de las variantes (para no chocar con las del
/// panel).
const FIRST_RUN: RunId = 10_000;

/// En qué punto está una variante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariantState {
    Running,
    /// Lista, con su `anim.js`.
    Ready(String),
    Failed(String),
}

/// Una variante.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variant {
    /// A, B, C…
    pub letter: char,
    pub state: VariantState,
    /// Lo que dijo el agente al terminar (qué hace esta variante).
    pub summary: String,
    /// Es una mezcla de las demás.
    pub mix: bool,
}

impl Variant {
    pub fn source(&self) -> Option<&str> {
        match &self.state {
            VariantState::Ready(source) => Some(source),
            _ => None,
        }
    }
}

/// Las variantes en marcha o listas.
pub struct Variants {
    store: Arc<dyn DocumentStore>,
    settings: Arc<dyn SettingsStore>,
    agents: Arc<dyn ArtAgents>,
    list: Arc<Mutex<Vec<Variant>>>,
    next_run: Mutex<RunId>,
    cancel: Mutex<CancellationToken>,
    notify: Arc<Mutex<Option<ArtNotify>>>,
}

fn ping(notify: &Mutex<Option<ArtNotify>>) {
    let callback = notify.lock().ok().and_then(|n| n.clone());
    if let Some(callback) = callback {
        callback();
    }
}

/// El encargo de una variante.
pub fn variant_goal(letter: char, count: usize, prompt: &str, action: Option<&str>) -> String {
    let focus = action
        .map(|a| format!("Céntrate en la acción «{a}»; deja las demás como están.\n"))
        .unwrap_or_default();
    crate::refine::fill(
        VARIANT_GOAL,
        &[
            ("letter", &letter.to_string()),
            ("count", &count.to_string()),
            ("prompt", prompt.trim()),
            ("focus", &focus),
        ],
    )
}

/// Lo que dijo el agente, recortado para la tarjeta.
fn summary_of(text: &str) -> String {
    let text = text.replace("<promise>LISTO</promise>", "");
    let text = text.trim();
    let first: String = text.chars().take(400).collect();
    if first.len() < text.len() {
        format!("{first}…")
    } else {
        first
    }
}

impl Variants {
    pub fn new(
        store: Arc<dyn DocumentStore>,
        settings: Arc<dyn SettingsStore>,
        agents: Arc<dyn ArtAgents>,
    ) -> Self {
        Self {
            store,
            settings,
            agents,
            list: Arc::default(),
            next_run: Mutex::new(FIRST_RUN),
            cancel: Mutex::new(CancellationToken::new()),
            notify: Arc::default(),
        }
    }

    /// Llama a `notify` cuando cambia algo (la ventana se repinta).
    pub fn set_notify(&self, notify: ArtNotify) {
        if let Ok(mut slot) = self.notify.lock() {
            *slot = Some(notify);
        }
    }

    /// Las variantes, en orden.
    pub fn list(&self) -> Vec<Variant> {
        self.list.lock().map(|l| l.clone()).unwrap_or_default()
    }

    /// ¿Queda alguna en marcha?
    pub fn running(&self) -> bool {
        self.list().iter().any(|v| v.state == VariantState::Running)
    }

    /// Para las que estén en marcha y las olvida todas.
    pub fn clear(&self) {
        if let Ok(cancel) = self.cancel.lock() {
            cancel.cancel();
        }
        if let Ok(mut list) = self.list.lock() {
            list.clear();
        }
        ping(&self.notify);
    }

    /// Empieza `count` variantes de la animación de `document` (sustituye a
    /// las que hubiera).
    pub async fn start(
        &self,
        document: &Document,
        prompt: &str,
        count: usize,
        action: Option<&str>,
    ) -> Result<(), PortError> {
        if document.animation.is_none() {
            return Err(PortError::Invalid(
                "el dibujo aún no tiene animación".into(),
            ));
        }
        if prompt.trim().is_empty() {
            return Err(PortError::Invalid("di qué variantes quieres".into()));
        }
        self.clear();
        let cancel = CancellationToken::new();
        if let Ok(mut slot) = self.cancel.lock() {
            *slot = cancel.clone();
        }
        let count = count.clamp(1, 8);
        for i in 0..count {
            let letter = (b'A' + i as u8) as char;
            let goal = variant_goal(letter, count, prompt, action);
            self.launch(document, letter, goal, false, cancel.clone())
                .await?;
        }
        Ok(())
    }

    /// Una variante más que mezcla las listas según `text`.
    pub async fn mix(&self, document: &Document, text: &str) -> Result<(), PortError> {
        let ready: Vec<(char, String)> = self
            .list()
            .iter()
            .filter_map(|v| v.source().map(|s| (v.letter, s.to_string())))
            .collect();
        if ready.is_empty() {
            return Err(PortError::Invalid(
                "aún no hay variantes listas para mezclar".into(),
            ));
        }
        if text.trim().is_empty() {
            return Err(PortError::Invalid("di qué quieres mezclar".into()));
        }
        let letter = (b'A' + self.list().len() as u8) as char;
        let mut goal = format!(
            "Haz la variante {letter}: una mezcla de las anteriores.\n\nQué mezclar: {}\n\nCambia solo la animación: escribe el anim.js resultante y compruébalo. No dibujes en el lienzo. Al terminar, resume en una frase qué tomaste de cada una.\n",
            text.trim()
        );
        for (l, source) in ready {
            goal.push_str(&format!("\nVariante {l}:\n```js\n{source}\n```\n"));
        }
        let cancel = self.cancel.lock().map(|c| c.clone()).unwrap_or_default();
        self.launch(document, letter, goal, true, cancel).await
    }

    async fn launch(
        &self,
        document: &Document,
        letter: char,
        goal: String,
        mix: bool,
        cancel: CancellationToken,
    ) -> Result<(), PortError> {
        let run = {
            let mut next = self.next_run.lock().map_err(|_| poisoned())?;
            *next += 1;
            *next
        };
        let animation = document
            .animation
            .clone()
            .ok_or_else(|| PortError::Invalid("el dibujo aún no tiene animación".into()))?;
        let base = animation
            .base
            .clone()
            .unwrap_or_else(|| Image::from_canvas(document.canvas()));
        let base_png = self
            .store
            .encode_image(&base)
            .map_err(|e| PortError::Invalid(e.message))?;
        let settings = self.settings.load()?;
        let session = Session::spawn(document.clone(), self.store.clone());
        let agent = self.agents.agent(session.handle.clone(), run).await?;
        let prepared = agent.backend.prepare(&settings, run)?;
        let task = AgentTask {
            run,
            goal,
            context: String::new(),
            attachments: Vec::new(),
            model: prepared.model.clone(),
            limits: AgentLimits::default(),
            canvas: (document.canvas().width(), document.canvas().height()),
            palette: document.palette.clone().filter(|p| p.name.is_some()),
            continuing: true,
            resume: None,
            anim: Some(AnimTask {
                source: animation.source.clone(),
                base_png,
            }),
            vector: None,
        };
        let index = {
            let mut list = self.list.lock().map_err(|_| poisoned())?;
            list.push(Variant {
                letter,
                state: VariantState::Running,
                summary: String::new(),
                mix,
            });
            list.len() - 1
        };
        ping(&self.notify);
        let list = self.list.clone();
        let notify = self.notify.clone();
        let original = animation.source;
        tokio::spawn(async move {
            let outcome = prepared
                .runner
                .run(task, prepared.tools, EventSink::channel().0, cancel)
                .await;
            let source = session
                .handle
                .snapshot()
                .await
                .ok()
                .and_then(|d| d.animation.map(|a| a.source));
            let (state, summary) = match (outcome, source) {
                (Err(e), _) => (VariantState::Failed(e.to_string()), String::new()),
                (Ok(summary), Some(source)) if source != original => {
                    (VariantState::Ready(source), summary_of(&summary.final_text))
                }
                (Ok(summary), _) => (
                    VariantState::Failed("el agente no cambió anim.js".into()),
                    summary_of(&summary.final_text),
                ),
            };
            if let Ok(mut list) = list.lock()
                && let Some(variant) = list.get_mut(index)
                && variant.letter == letter
            {
                variant.state = state;
                variant.summary = summary;
            }
            drop(agent);
            session.close().await;
            ping(&notify);
        });
        Ok(())
    }
}

fn poisoned() -> PortError {
    PortError::Unavailable("el estado de las variantes se rompió".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_goal_names_the_variant_and_the_action() {
        let goal = variant_goal('B', 3, " más rebote ", Some("andar"));
        assert!(goal.starts_with("Haz la variante B de 3"), "{goal}");
        assert!(goal.contains("Petición: más rebote\n"), "{goal}");
        assert!(goal.contains("Céntrate en la acción «andar»"), "{goal}");
        assert!(!variant_goal('A', 2, "x", None).contains("Céntrate"));
        assert_eq!(summary_of("Hecho.\n<promise>LISTO</promise>"), "Hecho.");
    }
}
