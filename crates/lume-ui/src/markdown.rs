//! Markdown sencillo para la conversación del agente: párrafos, títulos,
//! listas, bloques de código, **negrita**, *cursiva*, `código` y enlaces
//! (solo el texto). Suficiente para lo que escriben los modelos, sin
//! dependencias.

use eframe::egui::{
    self, Color32, CornerRadius, FontId, Margin, Stroke, TextFormat, text::LayoutJob,
};

use crate::theme;

/// Estilo base del texto.
#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub size: f32,
    pub color: Color32,
    /// Todo en cursiva (el razonamiento del agente).
    pub italic: bool,
}

/// Dibuja `text` como markdown.
pub fn show(ui: &mut egui::Ui, text: &str, style: Style) {
    ui.spacing_mut().item_spacing.y = 6.0;
    let mut lines = text.lines().peekable();
    let mut paragraph: Vec<&str> = Vec::new();
    let flush = |ui: &mut egui::Ui, paragraph: &mut Vec<&str>| {
        if !paragraph.is_empty() {
            let joined = paragraph.join(" ");
            ui.add(egui::Label::new(inline(&joined, style, style.size)).wrap());
            paragraph.clear();
        }
    };
    while let Some(line) = lines.next() {
        let trimmed = line.trim();
        // Bloque de código.
        if let Some(lang) = trimmed.strip_prefix("```") {
            flush(ui, &mut paragraph);
            let _ = lang;
            let mut code = Vec::new();
            for l in lines.by_ref() {
                if l.trim_start().starts_with("```") {
                    break;
                }
                code.push(l);
            }
            code_block(ui, &code.join("\n"));
            continue;
        }
        if trimmed.is_empty() {
            flush(ui, &mut paragraph);
            continue;
        }
        // Títulos.
        let hashes = trimmed.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && trimmed[hashes..].starts_with(' ') {
            flush(ui, &mut paragraph);
            let size = style.size + (4 - hashes.min(3)) as f32 * 1.5;
            let mut job = inline(trimmed[hashes..].trim(), style, size);
            for section in &mut job.sections {
                section.format.font_id = theme::semibold(size);
                section.format.color = theme::TEXT;
            }
            ui.add(egui::Label::new(job).wrap());
            continue;
        }
        // Listas.
        let indent = line.len() - line.trim_start().len();
        if let Some((marker, rest)) = list_item(trimmed) {
            flush(ui, &mut paragraph);
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.add_space(4.0 + (indent / 2) as f32 * 12.0);
                ui.label(
                    egui::RichText::new(marker)
                        .size(style.size)
                        .color(theme::TEXT_3),
                );
                ui.add(egui::Label::new(inline(rest, style, style.size)).wrap());
            });
            continue;
        }
        // Cita.
        if let Some(rest) = trimmed.strip_prefix('>') {
            flush(ui, &mut paragraph);
            let quoted = Style {
                color: theme::TEXT_3,
                ..style
            };
            ui.add(egui::Label::new(inline(rest.trim(), quoted, style.size)).wrap());
            continue;
        }
        paragraph.push(trimmed);
    }
    flush(ui, &mut paragraph);
}

/// `- algo`, `* algo`, `+ algo` → («•», «algo»); `3. algo` → («3.», «algo»).
fn list_item(line: &str) -> Option<(String, &str)> {
    for bullet in ["- ", "* ", "+ "] {
        if let Some(rest) = line.strip_prefix(bullet) {
            return Some(("•".into(), rest));
        }
    }
    let digits = line.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && line[digits..].starts_with(". ") {
        return Some((line[..digits + 1].to_string(), &line[digits + 2..]));
    }
    None
}

fn code_block(ui: &mut egui::Ui, code: &str) {
    egui::Frame::NONE
        .fill(theme::BG)
        .stroke(Stroke::new(1.0, theme::LINE_SOFT))
        .corner_radius(CornerRadius::same(6))
        .inner_margin(Margin::symmetric(10, 8))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.add(
                egui::Label::new(
                    egui::RichText::new(code)
                        .font(theme::mono(12.0))
                        .color(theme::TEXT_2),
                )
                .wrap(),
            );
        });
}

/// Una línea con negrita, cursiva, código y enlaces.
pub fn inline(text: &str, style: Style, size: f32) -> LayoutJob {
    let mut job = LayoutJob::default();
    let base = TextFormat {
        font_id: FontId::proportional(size),
        color: style.color,
        italics: style.italic,
        ..Default::default()
    };
    let (mut bold, mut italic) = (false, false);
    let mut buf = String::new();
    let push = |job: &mut LayoutJob, buf: &mut String, bold: bool, italic: bool| {
        if buf.is_empty() {
            return;
        }
        let mut format = base.clone();
        if bold {
            format.font_id = theme::semibold(size);
            format.color = theme::TEXT;
        }
        format.italics = style.italic || italic;
        job.append(buf, 0.0, format);
        buf.clear();
    };
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        // `código`
        if c == '`'
            && let Some(end) = chars[i + 1..].iter().position(|&x| x == '`')
        {
            push(&mut job, &mut buf, bold, italic);
            let code: String = chars[i + 1..i + 1 + end].iter().collect();
            job.append(
                &code,
                0.0,
                TextFormat {
                    font_id: theme::mono(size - 1.0),
                    color: theme::TEXT,
                    background: theme::HOVER,
                    ..Default::default()
                },
            );
            i += end + 2;
            continue;
        }
        // **negrita** / __negrita__
        if (c == '*' || c == '_') && next == Some(c) {
            push(&mut job, &mut buf, bold, italic);
            bold = !bold;
            i += 2;
            continue;
        }
        // *cursiva* / _cursiva_ (no dentro de palabras con _)
        if (c == '*' || (c == '_' && !prev_is_word(&chars, i)))
            && next.is_some_and(|n| !n.is_whitespace() || italic)
        {
            push(&mut job, &mut buf, bold, italic);
            italic = !italic;
            i += 1;
            continue;
        }
        // [texto](enlace): solo el texto, subrayado.
        if c == '['
            && let Some(close) = chars[i + 1..].iter().position(|&x| x == ']')
            && chars.get(i + 2 + close) == Some(&'(')
            && let Some(paren) = chars[i + 3 + close..].iter().position(|&x| x == ')')
        {
            push(&mut job, &mut buf, bold, italic);
            let label: String = chars[i + 1..i + 1 + close].iter().collect();
            let mut format = base.clone();
            format.color = theme::ACCENT;
            format.underline = Stroke::new(1.0, theme::ACCENT);
            job.append(&label, 0.0, format);
            i += close + 4 + paren;
            continue;
        }
        buf.push(c);
        i += 1;
    }
    push(&mut job, &mut buf, bold, italic);
    job
}

fn prev_is_word(chars: &[char], i: usize) -> bool {
    i > 0 && chars[i - 1].is_alphanumeric()
}
