//! Encargos que llegan por MCP: cómo se nombran en la interfaz y el menú
//! que los lista (en la cabecera del studio y en el menú Agente).

use std::sync::Arc;

use eframe::egui;
use lume_app::{ArtId, ArtState, ArtStatus};
use lume_domain::Canvas;

use crate::intent::Intent;

/// Un encargo en las listas de la interfaz.
#[derive(Debug, Clone, PartialEq)]
pub struct JobItem {
    pub id: ArtId,
    pub name: String,
    pub state: ArtState,
    /// «vuelta 1 de 3», «terminado»…
    pub state_text: String,
    /// Su lienzo ahora y la revisión del documento (para la miniatura).
    pub canvas: Arc<Canvas>,
    pub revision: u64,
}

impl JobItem {
    /// «cofre_madera · vuelta 1 de 3».
    pub fn label(&self) -> String {
        format!("{} · {}", self.name, self.state_text)
    }
}

/// El estado de un encargo en pocas palabras.
pub fn state_text(status: &ArtStatus) -> String {
    match status.state {
        ArtState::Running if status.turn == 0 => "empezando".into(),
        ArtState::Running => format!("vuelta {} de {}", status.turn, status.max_turns),
        other => other.label().into(),
    }
}

/// Las entradas de la lista de encargos dentro de un menú.
pub fn menu_entries(
    ui: &mut egui::Ui,
    jobs: &[JobItem],
    viewing: Option<ArtId>,
    out: &mut Vec<Intent>,
) {
    if viewing.is_some() {
        if ui.button("Volver a mi dibujo").clicked() {
            out.push(Intent::OpenMain);
            ui.close();
        }
        ui.separator();
    }
    if jobs.is_empty() {
        ui.add_enabled(false, egui::Button::new("No hay encargos"));
        return;
    }
    for job in jobs {
        let selected = viewing == Some(job.id);
        if ui
            .add(egui::Button::selectable(selected, job.label()))
            .clicked()
        {
            out.push(Intent::OpenJob(job.id));
            ui.close();
        }
    }
}
