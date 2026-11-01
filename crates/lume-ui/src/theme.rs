//! Tema de LumeArt: los tokens de `docs/04-diseno-ui.md` (y
//! `docs/diseno/Sistema.dc.html`) como constantes, y [`apply`], que los
//! convierte en un `egui::Style` y registra Geist y Geist Mono.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Shadow,
    Stroke, TextStyle, Theme, Vec2, Visuals,
};

// --- Color -----------------------------------------------------------------

/// Fondo de la ventana y del área del lienzo.
pub const BG: Color32 = Color32::from_rgb(0x0B, 0x0D, 0x10);
/// Cabecera, barras laterales, paneles, barra de estado.
pub const BAR: Color32 = Color32::from_rgb(0x10, 0x13, 0x18);
/// Barra de opciones, línea de tiempo, campos hundidos.
pub const BAR_ALT: Color32 = Color32::from_rgb(0x0E, 0x11, 0x15);
/// Tarjetas.
pub const CARD: Color32 = Color32::from_rgb(0x12, 0x15, 0x1A);
/// Botones secundarios y campos.
pub const FIELD: Color32 = Color32::from_rgb(0x16, 0x1A, 0x20);
/// Hover y elemento activo neutro.
pub const HOVER: Color32 = Color32::from_rgb(0x1C, 0x20, 0x27);
/// Separadores de paneles.
pub const LINE: Color32 = Color32::from_rgb(0x1F, 0x24, 0x2C);
/// Bordes de campos y botones.
pub const LINE_STRONG: Color32 = Color32::from_rgb(0x2A, 0x30, 0x3A);
/// Separadores dentro de paneles.
pub const LINE_SOFT: Color32 = Color32::from_rgb(0x23, 0x28, 0x31);
/// Caja del bucle de refinado.
pub const LOOP_BOX: Color32 = Color32::from_rgb(0x14, 0x17, 0x1C);

/// Texto principal.
pub const TEXT: Color32 = Color32::from_rgb(0xE8, 0xEA, 0xED);
/// Texto de cuerpo en el agente.
pub const TEXT_BODY: Color32 = Color32::from_rgb(0xC9, 0xCE, 0xD6);
/// Secundario.
pub const TEXT_2: Color32 = Color32::from_rgb(0xA3, 0xAA, 0xB6);
/// Etiquetas, apagado.
pub const TEXT_3: Color32 = Color32::from_rgb(0x8C, 0x94, 0xA1);
/// Muy apagado (reglas, placeholders).
pub const TEXT_4: Color32 = Color32::from_rgb(0x6E, 0x76, 0x83);

/// Acento (verde de la marca): botón principal, activo, resaltes, bucle.
pub const ACCENT: Color32 = Color32::from_rgb(0x3D, 0xBE, 0x6B);
/// Texto sobre el acento.
pub const ACCENT_INK: Color32 = Color32::from_rgb(0x05, 0x21, 0x0F);
/// Acento al 14 %: fondo de herramienta activa, pastillas.
pub const ACCENT_SOFT: Color32 = Color32::from_rgba_unmultiplied_const(0x3D, 0xBE, 0x6B, 36);
/// Acento al 35 %: borde del paso en curso.
pub const ACCENT_RING: Color32 = Color32::from_rgba_unmultiplied_const(0x3D, 0xBE, 0x6B, 89);
/// Estado correcto (puntos de conexión).
pub const OK: Color32 = Color32::from_rgb(0x5B, 0xD6, 0x9A);
/// Información, «Próximamente», comandos en el código.
pub const INFO: Color32 = Color32::from_rgb(0x7A, 0xB8, 0xFF);
/// Avisos.
pub const WARN: Color32 = Color32::from_rgb(0xF5, 0xB8, 0x41);
/// Peligro (Detener): texto, fondo y borde.
pub const DANGER: Color32 = Color32::from_rgb(0xFF, 0x9A, 0x90);
pub const DANGER_BG: Color32 = Color32::from_rgb(0x1E, 0x16, 0x16);
pub const DANGER_LINE: Color32 = Color32::from_rgb(0x3A, 0x2A, 0x2A);
/// Papel cebolla: fotograma anterior y siguiente (al 35 %).
pub const ONION_PREV: Color32 = Color32::from_rgba_unmultiplied_const(0xFF, 0x7A, 0x8A, 89);
pub const ONION_NEXT: Color32 = Color32::from_rgba_unmultiplied_const(0x6B, 0xB7, 0xFF, 89);
/// Cuadros de transparencia del lienzo.
pub const CHECKER_A: Color32 = Color32::from_rgb(0x1A, 0x1E, 0x24);
pub const CHECKER_B: Color32 = Color32::from_rgb(0x21, 0x25, 0x2C);
/// Puntos del fondo alrededor del lienzo (1 px cada 16 px).
pub const DOT: Color32 = Color32::from_rgb(0x1A, 0x1E, 0x25);
/// Fondo claro de la vista previa.
pub const PREVIEW_LIGHT: Color32 = Color32::from_rgb(0xE9, 0xE4, 0xD8);

// --- Medidas ---------------------------------------------------------------

/// Escala de espaciado.
pub const SPACE: [f32; 6] = [4.0, 8.0, 12.0, 16.0, 24.0, 48.0];
/// Radios: campos y botones pequeños, botones, tarjetas pequeñas, tarjetas,
/// diálogos.
pub const RADIUS_FIELD: u8 = 6;
pub const RADIUS_BUTTON: u8 = 8;
pub const RADIUS_CARD_SMALL: u8 = 10;
pub const RADIUS_CARD: u8 = 12;
pub const RADIUS_DIALOG: u8 = 14;

/// Armazón (1440×900 de referencia).
pub const HEADER_HEIGHT: f32 = 40.0;
pub const TOOLBAR_WIDTH: f32 = 52.0;
pub const OPTIONS_HEIGHT: f32 = 44.0;
pub const AGENT_WIDTH: f32 = 392.0;
pub const STATUS_HEIGHT: f32 = 26.0;
pub const BOTTOM_PIXEL_HEIGHT: f32 = 132.0;
pub const BOTTOM_VECTOR_HEIGHT: f32 = 220.0;
pub const TOOL_BUTTON: f32 = 36.0;
/// Alto normal de botón y campo.
pub const CONTROL_HEIGHT: f32 = 28.0;
/// Tamaño de ventana de referencia y mínimo.
pub const WINDOW_SIZE: [f32; 2] = [1440.0, 900.0];
pub const WINDOW_MIN_SIZE: [f32; 2] = [1024.0, 640.0];

// --- Tipografía ------------------------------------------------------------

/// Familia de Geist SemiBold (600): títulos, etiquetas de panel, «LumeArt».
pub const SEMIBOLD: &str = "geist-semibold";

/// Estilos de texto propios, además de los de egui.
pub const SCREEN_TITLE: &str = "titulo-pantalla";
pub const CARD_TITLE: &str = "titulo-tarjeta";
pub const PANEL_LABEL: &str = "etiqueta-panel";

/// Fuente semibold de un tamaño.
pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(SEMIBOLD.into()))
}

/// Fuente mono de un tamaño.
pub fn mono(size: f32) -> FontId {
    FontId::monospace(size)
}

/// Geist, Geist SemiBold y Geist Mono embebidas, con las fuentes de egui como
/// respaldo para símbolos que Geist no tiene.
pub fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut add = |name: &str, bytes: &'static [u8]| {
        fonts
            .font_data
            .insert(name.to_string(), Arc::new(FontData::from_static(bytes)));
    };
    add(
        "geist",
        include_bytes!("../../../assets/fonts/Geist-Regular.ttf"),
    );
    add(
        SEMIBOLD,
        include_bytes!("../../../assets/fonts/Geist-SemiBold.ttf"),
    );
    add(
        "geist-mono",
        include_bytes!("../../../assets/fonts/GeistMono-Regular.ttf"),
    );

    let proportional = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let monospace = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();
    let with_first = |first: &str, rest: &[String]| {
        std::iter::once(first.to_string())
            .chain(rest.iter().cloned())
            .collect::<Vec<_>>()
    };
    fonts
        .families
        .insert(FontFamily::Proportional, with_first("geist", &proportional));
    fonts
        .families
        .insert(FontFamily::Monospace, with_first("geist-mono", &monospace));
    fonts.families.insert(
        FontFamily::Name(SEMIBOLD.into()),
        with_first(SEMIBOLD, &with_first("geist", &proportional)),
    );
    fonts
}

fn widget(fill: Color32, stroke: Color32, text: Color32, radius: u8) -> egui::style::WidgetVisuals {
    egui::style::WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(1.0, stroke),
        corner_radius: CornerRadius::same(radius),
        fg_stroke: Stroke::new(1.0, text),
        expansion: 0.0,
    }
}

/// Visuals oscuros con los colores de LumeArt.
pub fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.dark_mode = true;
    v.override_text_color = None;
    v.panel_fill = BAR;
    v.window_fill = CARD;
    v.window_stroke = Stroke::new(1.0, LINE);
    v.window_corner_radius = CornerRadius::same(RADIUS_DIALOG);
    v.menu_corner_radius = CornerRadius::same(RADIUS_BUTTON);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 24,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    v.popup_shadow = Shadow {
        offset: [0, 4],
        blur: 12,
        spread: 0,
        color: Color32::from_black_alpha(100),
    };
    v.faint_bg_color = CARD;
    v.extreme_bg_color = BAR_ALT;
    v.text_edit_bg_color = Some(BAR_ALT);
    v.code_bg_color = FIELD;
    v.hyperlink_color = INFO;
    v.warn_fg_color = WARN;
    v.error_fg_color = DANGER;
    v.selection.bg_fill = ACCENT_SOFT;
    v.selection.stroke = Stroke::new(1.0, ACCENT);
    v.widgets.noninteractive = widget(BAR, LINE, TEXT_2, RADIUS_FIELD);
    v.widgets.inactive = widget(FIELD, LINE_STRONG, TEXT, RADIUS_FIELD);
    v.widgets.hovered = widget(HOVER, LINE_STRONG, TEXT, RADIUS_FIELD);
    v.widgets.active = widget(ACCENT_SOFT, ACCENT, ACCENT, RADIUS_FIELD);
    v.widgets.open = widget(HOVER, LINE_STRONG, TEXT, RADIUS_FIELD);
    v.text_cursor.stroke = Stroke::new(2.0, ACCENT);
    v
}

/// Aplica el tema de LumeArt: estilo oscuro (también si el sistema está en
/// modo claro), espaciados, tamaños de texto y fuentes.
pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_theme(Theme::Dark);
    ctx.all_styles_mut(|style| {
        style.visuals = visuals();
        let s = &mut style.spacing;
        s.item_spacing = Vec2::new(SPACE[1], SPACE[1]);
        s.button_padding = Vec2::new(SPACE[2], SPACE[0]);
        s.interact_size = Vec2::new(CONTROL_HEIGHT, CONTROL_HEIGHT);
        s.window_margin = Margin::same(SPACE[3] as i8);
        s.menu_margin = Margin::same(SPACE[1] as i8);
        s.indent = SPACE[3];
        // Barras sólidas: reservan su hueco y nunca tapan el texto (las
        // flotantes de egui se dibujaban encima del contenido).
        s.scroll = egui::style::ScrollStyle {
            bar_width: 5.0,
            bar_inner_margin: 3.0,
            dormant_handle_opacity: 0.35,
            ..egui::style::ScrollStyle::solid()
        };

        let ts = &mut style.text_styles;
        ts.insert(TextStyle::Heading, semibold(22.0));
        ts.insert(TextStyle::Body, FontId::proportional(13.0));
        ts.insert(TextStyle::Button, FontId::proportional(13.0));
        ts.insert(TextStyle::Small, FontId::proportional(11.0));
        ts.insert(TextStyle::Monospace, mono(12.0));
        ts.insert(TextStyle::Name(SCREEN_TITLE.into()), semibold(22.0));
        ts.insert(TextStyle::Name(CARD_TITLE.into()), semibold(14.5));
        ts.insert(TextStyle::Name(PANEL_LABEL.into()), semibold(11.0));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_sets_colors_spacing_and_fonts() {
        let ctx = egui::Context::default();
        apply(&ctx);
        // Aunque el sistema prefiera el modo claro, el tema es el oscuro.
        assert_eq!(ctx.theme(), Theme::Dark);
        let style = ctx.global_style();
        let v = &style.visuals;
        assert!(v.dark_mode);
        assert_eq!(v.panel_fill, BAR);
        assert_eq!(v.panel_fill, Color32::from_hex("#101318").unwrap());
        assert_eq!(v.window_fill, CARD);
        assert_eq!(v.extreme_bg_color, BAR_ALT);
        assert_eq!(v.selection.stroke.color, ACCENT);
        assert_eq!(v.widgets.inactive.bg_fill, FIELD);
        assert_eq!(v.widgets.inactive.bg_stroke.color, LINE_STRONG);
        assert_eq!(v.widgets.hovered.bg_fill, HOVER);
        assert_eq!(v.widgets.inactive.corner_radius, CornerRadius::same(6));
        assert_eq!(v.window_corner_radius, CornerRadius::same(14));
        assert_eq!(v.hyperlink_color, INFO);

        let s = &style.spacing;
        assert_eq!(s.item_spacing, Vec2::splat(8.0));
        assert_eq!(s.interact_size.y, 28.0);
        assert_eq!(s.button_padding, Vec2::new(12.0, 4.0));

        assert_eq!(
            style.text_styles[&TextStyle::Body],
            FontId::proportional(13.0)
        );
        assert_eq!(style.text_styles[&TextStyle::Heading].size, 22.0);
        assert_eq!(
            style.text_styles[&TextStyle::Name(CARD_TITLE.into())],
            semibold(14.5)
        );
        assert_eq!(style.text_styles[&TextStyle::Monospace], mono(12.0));
    }

    #[test]
    fn design_tokens_match_the_doc() {
        let hex = |c: Color32| c.to_hex();
        assert_eq!(hex(BG), "#0b0d10ff");
        assert_eq!(hex(ACCENT), "#3dbe6bff");
        assert_eq!(hex(ACCENT_INK), "#05210fff");
        assert_eq!(hex(TEXT), "#e8eaedff");
        assert_eq!(hex(LINE), "#1f242cff");
        assert_eq!(hex(DANGER), "#ff9a90ff");
        // Acento al 14 % y al 35 %.
        assert_eq!(ACCENT_SOFT.a(), (0.14_f32 * 255.0).round() as u8);
        assert_eq!(ACCENT_RING.a(), (0.35_f32 * 255.0).round() as u8);
        assert_eq!(SPACE, [4.0, 8.0, 12.0, 16.0, 24.0, 48.0]);
    }

    #[test]
    fn geist_fonts_are_embedded_first() {
        let fonts = fonts();
        assert_eq!(fonts.families[&FontFamily::Proportional][0], "geist");
        assert_eq!(fonts.families[&FontFamily::Monospace][0], "geist-mono");
        assert_eq!(
            fonts.families[&FontFamily::Name(SEMIBOLD.into())][0],
            SEMIBOLD
        );
        for name in ["geist", SEMIBOLD, "geist-mono"] {
            assert!(
                fonts.font_data[name].font.as_ref().as_ref().len() > 50_000,
                "{name}"
            );
        }
        // Las de egui quedan de respaldo (símbolos, emoji).
        assert!(fonts.families[&FontFamily::Proportional].len() > 1);
    }
}
