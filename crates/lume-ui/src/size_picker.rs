//! Elegir el tamaño de un documento nuevo: proporción (libre, 1:1, 16:9…),
//! ancho y alto. Con una proporción elegida, cambiar el ancho ajusta el alto
//! (y al revés). Lo usan «Nuevo pixel art» y «Nuevo dibujo vectorial».

use eframe::egui::{
    self, Color32, CornerRadius, FontId, RichText, Sense, Stroke, StrokeKind, vec2,
};

use crate::theme;

/// Proporciones que se ofrecen (`None`: libre).
pub const RATIOS: [(&str, Option<(u32, u32)>); 8] = [
    ("Libre", None),
    ("1:1", Some((1, 1))),
    ("4:3", Some((4, 3))),
    ("3:4", Some((3, 4))),
    ("16:9", Some((16, 9))),
    ("9:16", Some((9, 16))),
    ("3:2", Some((3, 2))),
    ("2:3", Some((2, 3))),
];

/// El tamaño que se está eligiendo (los campos son texto mientras se
/// escriben).
#[derive(Debug, Clone, PartialEq)]
pub struct SizeChoice {
    pub width: String,
    pub height: String,
    pub ratio: Option<(u32, u32)>,
    /// Lado máximo.
    pub max: u32,
}

impl Default for SizeChoice {
    /// El dibujo vectorial de siempre: 1024 × 1024.
    fn default() -> Self {
        Self::new(1024, 1024, MAX_VECTOR_SIZE)
    }
}

/// Lado máximo de un dibujo vectorial.
pub const MAX_VECTOR_SIZE: u32 = 4096;

/// Tamaños habituales de un dibujo vectorial.
pub const VECTOR_PRESETS: [(u32, u32); 6] = [
    (512, 512),
    (1024, 1024),
    (2048, 2048),
    (1024, 768),
    (1920, 1080),
    (1080, 1920),
];

impl SizeChoice {
    pub fn new(width: u32, height: u32, max: u32) -> Self {
        let ratio = RATIOS
            .iter()
            .filter_map(|(_, r)| *r)
            .find(|&(a, b)| width * b == height * a);
        Self {
            width: width.to_string(),
            height: height.to_string(),
            ratio,
            max,
        }
    }

    /// El tamaño elegido o por qué no vale.
    pub fn size(&self) -> Result<(u32, u32), String> {
        let parse = |s: &str| {
            s.trim()
                .parse::<u32>()
                .ok()
                .filter(|v| (1..=self.max).contains(v))
        };
        match (parse(&self.width), parse(&self.height)) {
            (Some(w), Some(h)) => Ok((w, h)),
            _ => Err(format!(
                "El ancho y el alto deben ser números enteros entre 1 y {}",
                self.max
            )),
        }
    }

    /// Pone un tamaño (por ejemplo, de una casilla) y se queda con su
    /// proporción si es una de las de la lista.
    pub fn set(&mut self, width: u32, height: u32) {
        *self = Self::new(width, height, self.max);
    }

    /// Con la proporción elegida, el alto que va con el ancho (o al revés).
    fn follow(&mut self, from_width: bool) {
        let Some((a, b)) = self.ratio else {
            return;
        };
        let round = |v: f64| (v.round() as u32).clamp(1, self.max);
        if from_width {
            if let Ok(w) = self.width.trim().parse::<u32>() {
                self.height = round(f64::from(w) * f64::from(b) / f64::from(a)).to_string();
            }
        } else if let Ok(h) = self.height.trim().parse::<u32>() {
            self.width = round(f64::from(h) * f64::from(a) / f64::from(b)).to_string();
        }
    }

    /// Las proporciones y los campos de ancho y alto.
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Proporción").size(12.5).color(theme::TEXT_2));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(6.0, 6.0);
            for (name, ratio) in RATIOS {
                if chip(ui, name, self.ratio == ratio).clicked() {
                    self.ratio = ratio;
                    self.follow(true);
                }
            }
        });
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            let label = ui.label(RichText::new("Ancho").size(12.5).color(theme::TEXT_2));
            let width = ui
                .add(
                    egui::TextEdit::singleline(&mut self.width)
                        .desired_width(60.0)
                        .margin(vec2(8.0, 5.0))
                        .font(theme::mono(12.5)),
                )
                .labelled_by(label.id);
            if width.changed() {
                self.follow(true);
            }
            ui.label(RichText::new("×").color(theme::TEXT_3));
            let label = ui.label(RichText::new("Alto").size(12.5).color(theme::TEXT_2));
            let height = ui
                .add(
                    egui::TextEdit::singleline(&mut self.height)
                        .desired_width(60.0)
                        .margin(vec2(8.0, 5.0))
                        .font(theme::mono(12.5)),
                )
                .labelled_by(label.id);
            if height.changed() {
                self.follow(false);
            }
        });
    }
}

/// Casilla compacta con texto en mono (proporción o tamaño).
pub fn chip(ui: &mut egui::Ui, label: &str, selected: bool) -> egui::Response {
    let galley =
        ui.painter()
            .layout_no_wrap(label.to_string(), FontId::monospace(12.0), theme::TEXT);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 20.0, 28.0), Sense::click());
    response.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, label));
    let painter = ui.painter();
    let (fill, stroke, text) = if selected {
        (theme::ACCENT_SOFT, theme::ACCENT, theme::TEXT)
    } else if response.hovered() {
        (theme::HOVER, theme::LINE_STRONG, theme::TEXT)
    } else {
        (Color32::TRANSPARENT, theme::LINE_SOFT, theme::TEXT_2)
    };
    painter.rect_filled(rect, CornerRadius::same(theme::RADIUS_FIELD), fill);
    painter.rect_stroke(
        rect,
        CornerRadius::same(theme::RADIUS_FIELD),
        Stroke::new(1.0, stroke),
        StrokeKind::Inside,
    );
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        FontId::monospace(12.0),
        text,
    );
    response
}
