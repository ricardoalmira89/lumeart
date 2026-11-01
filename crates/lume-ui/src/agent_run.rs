//! `AgentSupervisor`: lanza el bucle de refinado ([`RefineLoop`]) en el
//! runtime, le pasa a la ventana sus eventos (despertándola en cada uno), le
//! manda los mensajes que escribe el usuario mientras dibuja y lo detiene.
//!
//! El bucle se monta con los ajustes del momento, así que cambiar el
//! proveedor o las opciones del bucle sirve para la siguiente ejecución.

use std::sync::Arc;
use std::sync::mpsc;

use eframe::egui;
use lume_app::ports::{Attachment, CancellationToken};
use lume_app::{
    AgentBackend, DocumentStore, LoopEvent, LoopMessages, LoopSink, LoopSummary, RefineLoop,
    SessionHandle, Settings,
};
use lume_domain::{Origin, RunId};

/// Lo que llega del bucle en marcha.
#[derive(Debug, Clone, PartialEq)]
pub enum RunUpdate {
    Loop(LoopEvent),
    /// Terminó (con éxito, por un límite, detenido o con error).
    Finished(LoopSummary),
}

struct Active {
    run: RunId,
    cancel: CancellationToken,
    messages: LoopMessages,
    updates: mpsc::Receiver<RunUpdate>,
}

/// Qué cambia el agente.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AgentWork {
    /// El lienzo de píxeles (dibujar desde cero o seguir).
    #[default]
    Drawing,
    /// La animación (`anim.js`).
    Animation,
    /// El dibujo vectorial (`dibujo.js`).
    Vector,
}

/// Número de la próxima ejecución (compartido por todas las pestañas).
static NEXT_RUN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Lleva el bucle del agente (uno cada vez).
pub struct AgentSupervisor {
    backend: Option<Arc<dyn AgentBackend>>,
    store: Arc<dyn DocumentStore>,
    runtime: tokio::runtime::Handle,
    session: SessionHandle,
    ctx: egui::Context,
    active: Option<Active>,
}

impl AgentSupervisor {
    pub fn new(
        backend: Option<Arc<dyn AgentBackend>>,
        store: Arc<dyn DocumentStore>,
        runtime: tokio::runtime::Handle,
        session: SessionHandle,
        ctx: egui::Context,
    ) -> Self {
        Self {
            backend,
            store,
            runtime,
            session,
            ctx,
            active: None,
        }
    }

    /// ¿Tiene agente montado? (Las pestañas con sesión propia lo montan al
    /// pedir algo.)
    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// La sesión en la que dibuja.
    pub fn session(&self) -> &SessionHandle {
        &self.session
    }

    /// ¿Hay un bucle en marcha?
    pub fn running(&self) -> bool {
        self.active.is_some()
    }

    /// Número de la ejecución en marcha.
    pub fn current_run(&self) -> Option<RunId> {
        self.active.as_ref().map(|a| a.run)
    }

    /// Empieza el bucle con los ajustes de `settings`. Si no se sigue el
    /// dibujo y el lienzo tiene algo, el agente empieza borrándolo (un paso
    /// de deshacer). Con `animation`, el trabajo es cambiar `anim.js` (el
    /// lienzo es la base y no se borra).
    pub fn start(
        &mut self,
        settings: &Settings,
        goal: &str,
        attachments: Vec<Attachment>,
        continuing: bool,
        work: AgentWork,
    ) -> Result<RunId, String> {
        if self.active.is_some() {
            return Err("El agente ya está dibujando: detenlo antes de pedir otra cosa.".into());
        }
        let Some(backend) = self.backend.clone() else {
            return Err("Esta ventana no tiene agente.".into());
        };
        // Números únicos en toda la ventana (cada pestaña tiene el suyo).
        let run = NEXT_RUN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let vm = self.session.view().borrow().clone();
        let has_drawing = vm.canvas.pixels().iter().any(|p| p.a > 0);
        let clear_first = has_drawing && !continuing && work == AgentWork::Drawing;
        let mut refine = RefineLoop::new(
            backend,
            self.session.clone(),
            self.store.clone(),
            settings.clone(),
        );
        refine.animation = work == AgentWork::Animation;
        refine.vector = work == AgentWork::Vector;
        let messages = refine.messages();
        let (tx, updates) = mpsc::channel();
        let cancel = CancellationToken::new();
        let token = cancel.clone();
        let ctx = self.ctx.clone();
        let session = self.session.clone();
        let goal = match work {
            AgentWork::Animation => lume_app::anim_goal(goal),
            AgentWork::Drawing | AgentWork::Vector => goal.to_string(),
        };
        self.runtime.spawn(async move {
            if clear_first {
                let _ = session.line("clear", Origin::Agent { run }, None).await;
            }
            let (sink, mut events) = LoopSink::channel();
            let running = refine.run(run, &goal, attachments, sink, token);
            tokio::pin!(running);
            let summary = loop {
                tokio::select! {
                    event = events.recv() => {
                        if let Some(event) = event {
                            let _ = tx.send(RunUpdate::Loop(event));
                            ctx.request_repaint();
                        }
                    }
                    summary = &mut running => break summary,
                }
            };
            while let Ok(event) = events.try_recv() {
                let _ = tx.send(RunUpdate::Loop(event));
            }
            let _ = tx.send(RunUpdate::Finished(summary));
            ctx.request_repaint();
        });
        self.active = Some(Active {
            run,
            cancel,
            messages,
            updates,
        });
        Ok(run)
    }

    /// Manda un mensaje al bucle en marcha (entra en la vuelta siguiente).
    /// `false` si no hay bucle.
    pub fn send_message(&self, text: &str) -> bool {
        match &self.active {
            Some(active) => {
                active.messages.send(text);
                true
            }
            None => false,
        }
    }

    /// Pide parar el bucle (termina en su siguiente paso).
    pub fn stop(&self) {
        if let Some(active) = &self.active {
            active.cancel.cancel();
        }
    }

    /// Lo que ha llegado desde la última vez. Tras `Finished` ya no hay bucle
    /// en marcha.
    pub fn poll(&mut self) -> Vec<RunUpdate> {
        let Some(active) = &self.active else {
            return Vec::new();
        };
        let mut out: Vec<RunUpdate> = active.updates.try_iter().collect();
        let finished = out.iter().any(|u| matches!(u, RunUpdate::Finished(_)));
        // Si la tarea murió sin decir nada, se da por terminada.
        let gone = !finished
            && matches!(
                active.updates.try_recv(),
                Err(mpsc::TryRecvError::Disconnected)
            );
        if gone {
            out.push(RunUpdate::Finished(LoopSummary {
                turns: 0,
                versions: Vec::new(),
                usage: Default::default(),
                final_text: String::new(),
                end: lume_app::LoopEnd::Failed("la ejecución se interrumpió".into()),
                checks: Vec::new(),
                delivered: None,
                token_budget: 0,
            }));
        }
        if finished || gone {
            self.active = None;
        }
        out
    }
}
