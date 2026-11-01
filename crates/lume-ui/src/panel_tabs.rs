//! Las pestañas Código e Historial del panel derecho (modo pixel art).
//!
//! - **Código**: cada cambio del agente o de la CLI con sus líneas, como el
//!   registro de comandos del studio web de pxcli. Mientras el pincel pinta,
//!   la línea en curso va resaltada, las ya pintadas en su color y las que
//!   faltan apagadas.
//! - **Historial**: los pasos que se pueden deshacer, del más nuevo al más
//!   antiguo, con una miniatura de cómo quedó el dibujo en cada uno. Un clic
//!   vuelve a ese paso.

use std::sync::Arc;

use eframe::egui::{
    self, Align, Color32, CornerRadius, FontId, Rect, RichText, Sense, Stroke, StrokeKind,
    TextureHandle, TextureOptions, pos2, vec2,
};
use lume_app::{HistoryItem, ViewModel};
use lume_domain::{Canvas, Document, Origin};

use crate::agent_panel::{LineColors, command_line_colored};
use crate::brush::Brush;
use crate::intent::Intent;
use crate::shell::history_text;
use crate::theme;

/// Cambios que se guardan en la pestaña Código.
const MAX_BLOCKS: usize = 300;
/// Alto de cada fila de la pestaña Código.
const CODE_ROW: f32 = 22.0;
/// Alto de cada fila del historial.
const HISTORY_ROW: f32 = 52.0;
/// Lado de la miniatura del historial.
const THUMB: f32 = 40.0;

/// Un cambio con sus líneas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeBlock {
    pub revision: u64,
    pub origin: Origin,
    pub lines: Arc<Vec<String>>,
}

/// Cómo va una línea.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineState {
    Done,
    Now,
    Pending,
}

/// Lo que va a la pestaña Código.
#[derive(Debug, Clone, Default)]
pub struct CodeLog {
    pub blocks: Vec<CodeBlock>,
    /// Revisión del último `ViewModel` visto.
    seen: Option<u64>,
    /// Hasta qué línea se llevó la vista (para seguir al pincel sin
    /// impedir que el usuario mueva la lista).
    followed: Option<(u64, usize)>,
}

impl CodeLog {
    /// Apunta el cambio del `ViewModel` si es nuevo, del agente o de la CLI y
    /// trae líneas.
    pub fn observe(&mut self, vm: &ViewModel) {
        if self.seen == Some(vm.revision) {
            return;
        }
        self.seen = Some(vm.revision);
        let Some(change) = vm.last_change.as_ref() else {
            return;
        };
        let newest = self.blocks.last().map_or(0, |b| b.revision);
        if change.revision != vm.revision
            || change.revision <= newest
            || change.origin == Origin::User
            || change.lines.is_empty()
        {
            return;
        }
        self.blocks.push(CodeBlock {
            revision: change.revision,
            origin: change.origin,
            lines: change.lines.clone(),
        });
        if self.blocks.len() > MAX_BLOCKS {
            let extra = self.blocks.len() - MAX_BLOCKS;
            self.blocks.drain(..extra);
        }
    }

    /// Estado de la línea `line` del bloque `block` según el pincel.
    pub fn state(&self, block: &CodeBlock, line: usize, brush: &Brush) -> LineState {
        match brush.progress() {
            Some(p) if p.revision == block.revision => match line.cmp(&p.line) {
                std::cmp::Ordering::Less => LineState::Done,
                std::cmp::Ordering::Equal => LineState::Now,
                std::cmp::Ordering::Greater => LineState::Pending,
            },
            _ if brush.is_pending(block.revision) => LineState::Pending,
            _ => LineState::Done,
        }
    }
}

/// Una fila de la pestaña Código: la cabecera de un bloque, una línea o un
/// hueco entre bloques.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CodeRow {
    Header(usize),
    Line(usize, usize),
    Gap,
}

fn code_rows(log: &CodeLog) -> Vec<CodeRow> {
    let mut rows = Vec::new();
    for (b, block) in log.blocks.iter().enumerate() {
        if b > 0 {
            rows.push(CodeRow::Gap);
        }
        rows.push(CodeRow::Header(b));
        rows.extend((0..block.lines.len()).map(|l| CodeRow::Line(b, l)));
    }
    rows
}

/// Quién hizo un cambio, en minúsculas.
fn author(origin: Origin) -> &'static str {
    match origin {
        Origin::User => "tú",
        Origin::Cli => "cli",
        Origin::Agent { .. } => "agente",
    }
}

/// La pestaña Código.
pub fn code_tab(ui: &mut egui::Ui, log: &mut CodeLog, brush: &Brush) {
    if log.blocks.is_empty() {
        ui.add_space(8.0);
        ui.add(
            egui::Label::new(
                RichText::new(
                    "Aquí aparece el código que dibuja el agente, línea a línea, mientras lo pinta.",
                )
                .size(13.0)
                .color(theme::TEXT_3),
            )
            .wrap(),
        );
        return;
    }
    let rows = code_rows(log);
    // Seguir al pincel: llevar la vista a la línea en curso cuando cambia.
    let now = brush.progress().map(|p| (p.revision, p.line));
    let target = now
        .filter(|n| Some(*n) != log.followed)
        .and_then(|(rev, line)| {
            let b = log.blocks.iter().position(|b| b.revision == rev)?;
            rows.iter().position(|r| *r == CodeRow::Line(b, line))
        });
    log.followed = now.or(log.followed);
    let mut area = egui::ScrollArea::vertical()
        .id_salt("pestana-codigo")
        .auto_shrink([false, false]);
    if let Some(row) = target {
        let offset = (row as f32 * CODE_ROW - ui.available_height() / 2.0).max(0.0);
        area = area.vertical_scroll_offset(offset);
    } else if now.is_none() {
        area = area.stick_to_bottom(true);
    }
    area.show_rows(ui, CODE_ROW, rows.len(), |ui, range| {
        ui.spacing_mut().item_spacing.y = 0.0;
        for row in &rows[range] {
            let (rect, response) =
                ui.allocate_exact_size(vec2(ui.available_width(), CODE_ROW), Sense::hover());
            match *row {
                CodeRow::Gap => {}
                CodeRow::Header(b) => {
                    let block = &log.blocks[b];
                    let n = block.lines.len();
                    let plural = if n == 1 { "línea" } else { "líneas" };
                    let painting = brush.is_pending(block.revision);
                    let title = format!("{} · {n} {plural}", author(block.origin));
                    let (state, color) = if painting {
                        ("pintando…", theme::ACCENT)
                    } else {
                        ("hecho", theme::TEXT_4)
                    };
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::Label,
                            true,
                            format!("{title} · {state}"),
                        )
                    });
                    let painter = ui.painter();
                    painter.text(
                        rect.left_center(),
                        egui::Align2::LEFT_CENTER,
                        title,
                        theme::semibold(11.5),
                        theme::TEXT_3,
                    );
                    painter.text(
                        rect.right_center() - vec2(4.0, 0.0),
                        egui::Align2::RIGHT_CENTER,
                        state,
                        FontId::proportional(11.5),
                        color,
                    );
                }
                CodeRow::Line(b, l) => {
                    let block = &log.blocks[b];
                    let state = log.state(block, l, brush);
                    let said = match state {
                        LineState::Done => "pintada",
                        LineState::Now => "pintando",
                        LineState::Pending => "pendiente",
                    };
                    response.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::ListItem,
                            true,
                            format!("línea {}: {} ({said})", l + 1, block.lines[l]),
                        )
                    });
                    if state == LineState::Now {
                        ui.painter()
                            .rect_filled(rect, CornerRadius::same(4), theme::ACCENT_SOFT);
                    }
                    let colors = match state {
                        LineState::Done => LineColors::NORMAL,
                        LineState::Now => LineColors::NOW,
                        LineState::Pending => LineColors::MUTED,
                    };
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(rect.shrink2(vec2(6.0, 0.0)))
                            .layout(egui::Layout::left_to_right(Align::Center)),
                    );
                    row_ui.spacing_mut().item_spacing.x = 8.0;
                    row_ui.label(
                        RichText::new(format!("{:>3}", l + 1))
                            .font(theme::mono(11.0))
                            .color(theme::TEXT_4),
                    );
                    command_line_colored(&mut row_ui, &block.lines[l], colors);
                }
            }
        }
    });
}

/// Miniaturas del historial: cómo quedó el dibujo tras cada paso.
#[derive(Default)]
pub struct HistoryThumbs {
    /// Una por paso, en el orden del historial.
    items: Vec<(HistoryItem, Option<Arc<Canvas>>)>,
    textures: Vec<Option<TextureHandle>>,
    seen: Option<u64>,
    /// Revisión en la que ya se pidió el documento (una vez por revisión).
    filled: Option<u64>,
}

impl Clone for HistoryThumbs {
    fn clone(&self) -> Self {
        // Las texturas se vuelven a crear al dibujar.
        Self {
            items: self.items.clone(),
            textures: Vec::new(),
            seen: self.seen,
            filled: self.filled,
        }
    }
}

impl std::fmt::Debug for HistoryThumbs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HistoryThumbs")
            .field("items", &self.items.len())
            .finish_non_exhaustive()
    }
}

impl HistoryThumbs {
    /// Sigue el historial del `ViewModel`: los pasos que siguen igual
    /// conservan su miniatura y el último siempre es el lienzo actual.
    pub fn observe(&mut self, vm: &ViewModel) {
        if self.seen == Some(vm.revision) {
            return;
        }
        self.seen = Some(vm.revision);
        let same = self
            .items
            .iter()
            .zip(&vm.history)
            .take_while(|((a, _), b)| a == *b)
            .count();
        self.items.truncate(same);
        self.textures.truncate(same);
        for item in &vm.history[same..] {
            self.items.push((item.clone(), None));
        }
        if let Some(last) = self.items.last_mut() {
            last.1 = Some(vm.canvas.clone());
            if let Some(t) = self.textures.get_mut(self.items.len() - 1) {
                *t = None;
            }
        }
    }

    /// ¿Faltan miniaturas de pasos anteriores (al abrir un archivo con
    /// historial, por ejemplo)? Entonces hay que llamar a [`Self::fill`].
    pub fn needs_document(&self) -> bool {
        self.filled != self.seen && self.items.iter().any(|(_, c)| c.is_none())
    }

    /// Rellena las que faltan con el documento: desde el lienzo actual, se
    /// deshace paso a paso hacia atrás.
    pub fn fill(&mut self, document: &Document) {
        self.filled = self.seen;
        let entries = document.history().entries();
        if entries.len() != self.items.len() {
            return;
        }
        let mut canvas = document.canvas().clone();
        for i in (0..entries.len()).rev() {
            if self.items[i].1.is_none() {
                self.items[i].1 = Some(Arc::new(canvas.clone()));
            }
            if entries[i].patch.revert(&mut canvas).is_err() {
                break;
            }
        }
    }

    /// Miniatura del paso `i` (`None` si aún no se sabe).
    pub fn canvas(&self, i: usize) -> Option<&Arc<Canvas>> {
        self.items.get(i).and_then(|(_, c)| c.as_ref())
    }

    fn texture(&mut self, ctx: &egui::Context, i: usize) -> Option<egui::TextureId> {
        if self.textures.len() < self.items.len() {
            self.textures.resize(self.items.len(), None);
        }
        if self.textures[i].is_none() {
            let canvas = self.items[i].1.as_ref()?;
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [canvas.width() as usize, canvas.height() as usize],
                &canvas.to_rgba_bytes(),
            );
            self.textures[i] =
                Some(ctx.load_texture(format!("historial-{i}"), image, TextureOptions::NEAREST));
        }
        self.textures[i].as_ref().map(|t| t.id())
    }
}

/// La pestaña Historial: del paso más nuevo al más antiguo, con miniatura.
pub fn history_tab(
    ui: &mut egui::Ui,
    vm: &ViewModel,
    thumbs: &mut HistoryThumbs,
    out: &mut Vec<Intent>,
) {
    let n = vm.history.len();
    if n == 0 {
        ui.add_space(8.0);
        ui.label(
            RichText::new("Sin cambios todavía")
                .size(13.0)
                .color(theme::TEXT_3),
        );
        return;
    }
    egui::ScrollArea::vertical()
        .id_salt("pestana-historial")
        .auto_shrink([false, false])
        .show_rows(ui, HISTORY_ROW, n, |ui, range| {
            ui.spacing_mut().item_spacing.y = 0.0;
            for row in range {
                // De arriba abajo: del más nuevo al más antiguo.
                let i = n - 1 - row;
                let item = &vm.history[i];
                let latest = i + 1 == n;
                let text = history_text(item);
                let who = author(item.origin);
                let (rect, response) =
                    ui.allocate_exact_size(vec2(ui.available_width(), HISTORY_ROW), Sense::click());
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{text} — {who}"))
                });
                let response = if latest {
                    response
                } else {
                    response
                        .on_hover_text("Volver a este paso")
                        .on_hover_cursor(egui::CursorIcon::PointingHand)
                };
                let texture = thumbs.texture(ui.ctx(), i);
                let size = thumbs
                    .canvas(i)
                    .map(|c| (c.width() as f32, c.height() as f32));
                let painter = ui.painter();
                if latest || response.hovered() {
                    painter.rect_filled(
                        rect.shrink2(vec2(0.0, 2.0)),
                        CornerRadius::same(6),
                        Color32::from_rgb(0x18, 0x1C, 0x22),
                    );
                }
                // Miniatura sobre cuadros, ajustada a un cuadrado de 40.
                let slot = Rect::from_center_size(
                    pos2(rect.left() + 8.0 + THUMB / 2.0, rect.center().y),
                    vec2(THUMB, THUMB),
                );
                painter.rect_filled(slot, CornerRadius::same(4), theme::CHECKER_A);
                if let (Some(texture), Some((w, h))) = (texture, size) {
                    let k = (THUMB / w.max(h)).min(THUMB);
                    let image = Rect::from_center_size(slot.center(), vec2(w, h) * k);
                    painter.image(
                        texture,
                        image,
                        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
                painter.rect_stroke(
                    slot,
                    CornerRadius::same(4),
                    Stroke::new(1.0, theme::LINE_STRONG),
                    StrokeKind::Outside,
                );
                let (text_color, who_color) = if latest {
                    (theme::TEXT, theme::ACCENT)
                } else {
                    (theme::TEXT_2, theme::TEXT_3)
                };
                let x = slot.right() + 12.0;
                painter.text(
                    pos2(x, rect.center().y - 8.0),
                    egui::Align2::LEFT_CENTER,
                    &text,
                    FontId::proportional(12.5),
                    text_color,
                );
                let detail = if latest {
                    format!("{who} · ahora")
                } else {
                    format!("{who} · hace {} {}", n - 1 - i, pasos(n - 1 - i))
                };
                painter.text(
                    pos2(x, rect.center().y + 9.0),
                    egui::Align2::LEFT_CENTER,
                    detail,
                    FontId::proportional(11.5),
                    who_color,
                );
                if response.clicked() && !latest {
                    out.push(Intent::UndoSteps(n - 1 - i));
                }
            }
        });
}

fn pasos(n: usize) -> &'static str {
    if n == 1 { "paso" } else { "pasos" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lume_app::{Change, DrawSpeed};
    use lume_domain::Patch;

    fn batch(doc: &mut Document, revision: u64, origin: Origin, lines: &[&str]) -> ViewModel {
        let before = doc.canvas().clone();
        let script = lines.join("\n");
        doc.run_batch(&script, origin, &|p: &str| p.to_string(), &mut |_, _, _| {
            Ok(())
        })
        .unwrap();
        let change = Change {
            revision,
            origin,
            label: "batch".into(),
            patch: Arc::new(Patch::between(&before, doc.canvas())),
            lines: Arc::new(lines.iter().map(|s| s.to_string()).collect()),
        };
        ViewModel::of(doc, revision, Some(change))
    }

    #[test]
    fn the_code_log_follows_the_brush() {
        let mut doc = Document::new("c", 8, 8).unwrap();
        let mut log = CodeLog::default();
        let mut brush = Brush::default();
        let vm = ViewModel::of(&doc, 0, None);
        log.observe(&vm);
        brush.observe(&vm, DrawSpeed::Normal);
        let vm = batch(
            &mut doc,
            1,
            Origin::Agent { run: 1 },
            &["fill_rect 0 0 8 2 #ff0000", "set_pixel 7 7 #00ff00"],
        );
        log.observe(&vm);
        brush.observe(&vm, DrawSpeed::Normal);
        assert_eq!(log.blocks.len(), 1);
        let block = log.blocks[0].clone();
        assert_eq!(log.state(&block, 0, &brush), LineState::Now);
        assert_eq!(log.state(&block, 1, &brush), LineState::Pending);
        // Una celda por fotograma (160 por segundo).
        while brush.progress().is_some_and(|p| p.line == 0) {
            brush.tick(1.0 / 160.0, DrawSpeed::Normal);
        }
        assert_eq!(log.state(&block, 0, &brush), LineState::Done);
        assert_eq!(log.state(&block, 1, &brush), LineState::Now);
        while brush.is_painting() {
            brush.tick(0.016, DrawSpeed::Normal);
        }
        assert_eq!(log.state(&block, 1, &brush), LineState::Done);

        // Lo que dibujas tú no va al código; el mismo ViewModel, una vez.
        let _ = doc.handle_line("set_pixel 1 1 #ffffff", Origin::User);
        log.observe(&ViewModel::of(&doc, 2, None));
        log.observe(&vm);
        assert_eq!(log.blocks.len(), 1);
    }

    #[test]
    fn thumbnails_keep_the_steps_that_did_not_change() {
        let mut doc = Document::new("h", 4, 4).unwrap();
        let mut thumbs = HistoryThumbs::default();
        let vm1 = batch(&mut doc, 1, Origin::Cli, &["set_pixel 0 0 #ff0000"]);
        thumbs.observe(&vm1);
        let vm2 = batch(&mut doc, 2, Origin::Cli, &["set_pixel 1 1 #00ff00"]);
        thumbs.observe(&vm2);
        assert!(!thumbs.needs_document());
        assert_eq!(thumbs.canvas(0).unwrap().get(1, 1).unwrap().a, 0);
        assert_eq!(thumbs.canvas(1).unwrap().get(1, 1).unwrap().g, 255);

        // Deshacer: queda el primero, que es el lienzo actual.
        let _ = doc.handle_line("undo", Origin::User);
        thumbs.observe(&ViewModel::of(&doc, 3, None));
        assert_eq!(thumbs.items.len(), 1);

        // Un historial que ya estaba (abrir un archivo): se rellena con el
        // documento.
        let mut fresh = HistoryThumbs::default();
        fresh.observe(&vm2);
        let _ = doc.handle_line("redo", Origin::User);
        let mut opened = HistoryThumbs::default();
        opened.observe(&ViewModel::of(&doc, 4, None));
        assert!(opened.needs_document());
        opened.fill(&doc);
        assert!(!opened.needs_document());
        assert_eq!(opened.canvas(0).unwrap().get(0, 0).unwrap().r, 255);
        assert_eq!(opened.canvas(0).unwrap().get(1, 1).unwrap().a, 0);
        assert_eq!(opened.canvas(1).unwrap().get(1, 1).unwrap().g, 255);
    }
}
