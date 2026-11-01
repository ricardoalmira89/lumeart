//! Tests del armazón del studio con `egui_kittest` (sin ventana).

use eframe::egui;
use egui_kittest::Harness;
use egui_kittest::kittest::{NodeT, Queryable};
use lume_app::ViewModel;
use lume_domain::{Document, Origin};
use lume_ui::{AppShell, Intent, theme};

/// Lo que el arnés le pasa al armazón en cada fotograma.
struct State {
    shell: AppShell,
    vm: ViewModel,
    intents: Vec<Intent>,
    themed: bool,
}

/// Un documento con algo dibujado y un paso que deshacer.
fn view_model() -> ViewModel {
    let mut doc = Document::new("pocion_morada", 32, 32).unwrap();
    for line in [
        "fill_rect 10 18 12 10 #8b3fe0",
        "fill_rect 13 3 6 3 #a86b3c",
        "line 12 6 19 6 #dceef6",
        "set_pixel 21 24 #5b2aa6",
    ] {
        let _ = doc.handle_line(line, Origin::Agent { run: 1 });
    }
    ViewModel::of(&doc, 4, None)
}

fn harness() -> Harness<'static, State> {
    let state = State {
        shell: AppShell::default(),
        vm: view_model(),
        intents: Vec::new(),
        themed: false,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .build_ui_state(
            |ui, state: &mut State| {
                if !state.themed {
                    theme::apply(ui.ctx());
                    state.themed = true;
                }
                // El arnés deja 8 px de margen; la ventana real no: el
                // armazón se dibuja sobre la pantalla entera.
                let screen = ui.ctx().content_rect();
                let intents = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(screen), |ui| {
                        ui.set_clip_rect(screen);
                        state.shell.show(ui, &state.vm)
                    })
                    .inner;
                state.intents.extend(intents);
            },
            state,
        );
    harness.run();
    harness
}

fn assert_near(a: f32, b: f32, what: &str) {
    assert!((a - b).abs() <= 0.5, "{what}: {a} != {b}");
}

#[test]
fn zones_have_the_sizes_of_the_design() {
    let harness = harness();
    let z = harness.state().shell.zones;
    // docs/04-diseno-ui.md · Armazón (1440×900).
    assert_near(z.header.height(), 40.0, "cabecera, alto");
    assert_near(z.header.width(), 1440.0, "cabecera, ancho");
    assert_near(z.header.top(), 0.0, "cabecera, arriba");
    assert_near(z.status.height(), 26.0, "estado, alto");
    assert_near(z.status.bottom(), 900.0, "estado, abajo");
    assert_near(z.agent.width(), 392.0, "agente, ancho");
    assert_near(z.agent.right(), 1440.0, "agente, derecha");
    assert_near(z.toolbar.width(), 52.0, "herramientas, ancho");
    assert_near(z.toolbar.left(), 0.0, "herramientas, izquierda");
    assert_near(z.options.height(), 44.0, "opciones, alto");
    assert_near(z.bottom.height(), 132.0, "panel inferior, alto");
    // Herramientas y agente van de la cabecera a la barra de estado.
    for (rect, what) in [(z.toolbar, "herramientas"), (z.agent, "agente")] {
        assert_near(rect.top(), 40.0, what);
        assert_near(rect.bottom(), 900.0 - 26.0, what);
    }
    // Opciones, lienzo y panel inferior llenan el centro, en ese orden.
    assert_near(z.options.left(), 52.0, "opciones, izquierda");
    assert_near(z.options.right(), 1440.0 - 392.0, "opciones, derecha");
    assert_near(
        z.canvas.top(),
        z.options.bottom(),
        "lienzo debajo de opciones",
    );
    assert_near(
        z.canvas.bottom(),
        z.bottom.top(),
        "lienzo encima del panel inferior",
    );
    assert_near(
        z.bottom.bottom(),
        900.0 - 26.0,
        "panel inferior sobre la barra de estado",
    );
    assert_near(z.canvas.width(), 1440.0 - 52.0 - 392.0, "lienzo, ancho");
    assert_near(
        z.canvas.height(),
        900.0 - 40.0 - 44.0 - 132.0 - 26.0,
        "lienzo, alto",
    );
}

/// Los tres modos están y se pueden elegir (Vector llegó con T:53).
#[test]
fn mode_selector_has_three_modes() {
    let harness = harness();
    for (label, enabled) in [("Pixel art", true), ("Vector", true), ("Animación", true)] {
        let node = harness.get_by_label(label);
        assert_eq!(!node.accesskit_node().is_disabled(), enabled, "{label}");
    }
}

#[test]
fn the_shell_only_emits_intents() {
    let mut harness = harness();
    let before = harness.state().vm.clone();
    assert!(harness.state().intents.is_empty());

    harness.get_by_label("Deshacer").click();
    harness.run();
    harness.get_by_label_contains("Exportar").click();
    harness.run();
    // Rehacer está deshabilitado (no hay nada que rehacer): no emite nada.
    harness.get_by_label("Rehacer").click();
    harness.run();

    assert_eq!(harness.state().intents, vec![Intent::Undo, Intent::Export]);
    // El armazón no tiene acceso a la sesión: el ViewModel sigue igual.
    assert_eq!(harness.state().vm, before);

    // Elegir una herramienta es estado de la interfaz, no un intent.
    harness.get_by_label("Relleno (G)").click();
    harness.run();
    assert_eq!(harness.state().shell.tool, lume_ui::Tool::Fill);
    assert_eq!(harness.state().intents.len(), 2);
}

/// Captura para que el usuario la compare con
/// `docs/diseno/capturas/01-studio-pixel.png`. Necesita GPU (wgpu); se genera
/// con `UPDATE_SNAPSHOTS=1 cargo test -p lume-ui --test shell -- --ignored`.
#[test]
#[ignore = "necesita GPU (wgpu) para renderizar"]
fn snapshot_for_review() {
    let state = State {
        shell: {
            let mut shell = AppShell::default();
            shell.server = Some("127.0.0.1:7777".into());
            shell.agent_status = Some("Vuelta 3 de 8".into());
            shell
        },
        vm: view_model(),
        intents: Vec::new(),
        themed: false,
    };
    let mut harness = Harness::builder()
        .with_size(egui::vec2(1440.0, 900.0))
        .wgpu()
        .build_ui_state(
            |ui, state: &mut State| {
                if !state.themed {
                    theme::apply(ui.ctx());
                    state.themed = true;
                }
                // El arnés deja 8 px de margen; la ventana real no: el
                // armazón se dibuja sobre la pantalla entera.
                let screen = ui.ctx().content_rect();
                let intents = ui
                    .scope_builder(egui::UiBuilder::new().max_rect(screen), |ui| {
                        ui.set_clip_rect(screen);
                        state.shell.show(ui, &state.vm)
                    })
                    .inner;
                state.intents.extend(intents);
            },
            state,
        );
    harness.run();
    harness.snapshot("studio-armazon");
}

/// La barra de estado muestra el píxel bajo el cursor, su color y el zoom.
#[test]
fn status_bar_shows_coordinates_color_and_zoom() {
    let mut harness = harness();
    let viewport = harness
        .state()
        .shell
        .canvas
        .viewport
        .expect("el lienzo se dibujó");
    // 32×32 en el área del diseño se ajusta a 1700 %.
    assert_eq!(viewport.percent(), 1700);
    let over = viewport.pixel_rect(21, 24).center();
    harness.hover_at(over);
    harness.run();

    assert_eq!(harness.state().shell.canvas.hover, Some((21, 24)));
    harness.get_by_label("x 21 · y 24");
    // En la barra de estado (la paleta también tiene una casilla con ese nombre).
    harness.get_by_role_and_label(egui::Role::Label, "#5B2AA6");
    assert!(
        harness.get_all_by_label("1700%").count() >= 2,
        "zoom en opciones y en estado"
    );

    // Sobre un píxel transparente, el color lleva alfa.
    harness.hover_at(viewport.pixel_rect(0, 0).center());
    harness.run();
    harness.get_by_label("x 0 · y 0");
    harness.get_by_label("#00000000");

    // Fuera del lienzo no hay coordenadas.
    harness.hover_at(viewport.pixel_rect(-3, -3).center());
    harness.run();
    assert_eq!(harness.state().shell.canvas.hover, None);
    harness.get_by_label("x – · y –");
}

/// Pulsar el zoom de la barra de opciones vuelve a ajustar el lienzo.
#[test]
fn zoom_button_fits_the_canvas() {
    let mut harness = harness();
    harness.state_mut().shell.canvas.zoom = 40.0;
    harness.run();
    harness
        .get_by_role_and_label(egui::Role::Button, "4000%")
        .click();
    harness.run();
    assert_eq!(harness.state().shell.canvas.zoom, 17.0);
}

/// La barra de opciones cambia con la herramienta activa.
#[test]
fn options_bar_shows_the_active_tool_options() {
    let mut harness = harness();
    // Lápiz (por defecto).
    harness.get_by_label("Píxel perfecto");
    harness.get_by_label("Simetría X");
    harness.get_by_label("Tamaño");
    assert!(harness.query_by_label("Con relleno").is_none());

    harness.get_by_label("Rectángulo (U)").click();
    harness.run();
    harness.get_by_label("Con relleno");
    assert!(harness.query_by_label("Píxel perfecto").is_none());

    harness.get_by_label("Relleno (G)").click();
    harness.run();
    harness.get_by_label_contains("4 vecinos");
    assert!(harness.query_by_label("Tamaño").is_none());

    harness.get_by_label("Cuentagotas (I)").click();
    harness.run();
    harness.get_by_label_contains("clic derecho");

    // Las opciones cambian el estado de la herramienta.
    harness.get_by_label("Rectángulo (U)").click();
    harness.run();
    harness.get_by_label("Con relleno").click();
    harness.run();
    assert!(harness.state().shell.options.filled);
}

fn press(
    harness: &mut Harness<'static, State>,
    pos: egui::Pos2,
    pressed: bool,
    button: egui::PointerButton,
) {
    harness.event(egui::Event::PointerMoved(pos));
    harness.event(egui::Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
    harness.run();
}

/// Un trazo con el ratón produce un único intent con su etiqueta.
#[test]
fn a_pencil_stroke_with_the_mouse_is_one_intent() {
    let mut harness = harness();
    let vp = harness.state().shell.canvas.viewport.unwrap();
    let primary = egui::PointerButton::Primary;
    press(&mut harness, vp.pixel_rect(2, 2).center(), true, primary);
    for x in 3..=6 {
        harness.event(egui::Event::PointerMoved(vp.pixel_rect(x, 2).center()));
        harness.run();
    }
    // Mientras se arrastra hay vista previa y aún no hay intent.
    assert!(harness.state().shell.machine.is_active());
    assert!(harness.state().intents.is_empty());
    press(&mut harness, vp.pixel_rect(6, 2).center(), false, primary);

    let intents = &harness.state().intents;
    assert_eq!(intents.len(), 1);
    match &intents[0] {
        Intent::Apply { command, label } => {
            assert_eq!(label, "Lápiz · 5 píxeles");
            assert_eq!(
                command,
                &lume_domain::Command::FillRect {
                    x: 2,
                    y: 2,
                    w: 5,
                    h: 1,
                    color: lume_domain::Rgba::rgb(0x8B, 0x3F, 0xE0)
                }
            );
        }
        other => panic!("{other:?}"),
    }
}

/// El cuentagotas cambia el color principal sin tocar el documento.
#[test]
fn eyedropper_sets_the_primary_color() {
    let mut harness = harness();
    harness.get_by_label("Cuentagotas (I)").click();
    harness.run();
    let vp = harness.state().shell.canvas.viewport.unwrap();
    let at = vp.pixel_rect(21, 24).center();
    press(&mut harness, at, true, egui::PointerButton::Primary);
    press(&mut harness, at, false, egui::PointerButton::Primary);
    assert_eq!(
        harness.state().shell.primary,
        lume_domain::Rgba::rgb(0x5B, 0x2A, 0xA6)
    );
    assert!(harness.state().intents.is_empty());
}

/// Los atajos eligen herramienta y Esc cancela un arrastre.
#[test]
fn shortcuts_and_escape() {
    let mut harness = harness();
    harness.key_press(egui::Key::L);
    harness.run();
    assert_eq!(harness.state().shell.tool, lume_ui::Tool::Line);
    let vp = harness.state().shell.canvas.viewport.unwrap();
    press(
        &mut harness,
        vp.pixel_rect(1, 1).center(),
        true,
        egui::PointerButton::Primary,
    );
    harness.event(egui::Event::PointerMoved(vp.pixel_rect(9, 5).center()));
    harness.run();
    assert!(harness.state().shell.machine.is_active());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(!harness.state().shell.machine.is_active());
    press(
        &mut harness,
        vp.pixel_rect(9, 5).center(),
        false,
        egui::PointerButton::Primary,
    );
    assert!(
        harness.state().intents.is_empty(),
        "el arrastre cancelado no aplica nada"
    );

    harness.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::Z);
    harness.run();
    assert_eq!(harness.state().intents, vec![Intent::Undo]);
}

/// Un atajo pulsado y soltado entero dentro de un fotograma (Ctrl abajo, tecla,
/// Ctrl arriba) sigue contando: al acabar el fotograma Ctrl ya no está pulsado,
/// pero el evento de la tecla llegó con Ctrl.
#[test]
fn a_shortcut_released_within_one_frame_still_counts() {
    let mut harness = harness();
    assert!(harness.state().vm.can_undo);
    for key in [egui::Key::Z, egui::Key::S] {
        let input = harness.input_mut();
        for (pressed, mods) in [
            (true, egui::Modifiers::COMMAND),
            (false, egui::Modifiers::NONE),
        ] {
            input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed,
                repeat: false,
                modifiers: mods,
            });
        }
        harness.step();
    }
    harness.run();
    assert_eq!(harness.state().intents, vec![Intent::Undo, Intent::Save]);
}

/// Si pulsar, mover y soltar llegan en el mismo fotograma (ratón rápido), la
/// herramienta recibe los eventos en orden: el rectángulo va del punto donde
/// se pulsó al punto donde se soltó.
#[test]
fn fast_events_in_one_frame_keep_their_order() {
    let mut harness = harness();
    harness.key_press(egui::Key::U);
    harness.run();
    let vp = harness.state().shell.canvas.viewport.unwrap();
    let (a, b) = (
        vp.pixel_rect(10, 10).center(),
        vp.pixel_rect(14, 13).center(),
    );
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    harness.event(egui::Event::PointerMoved(a));
    harness.event(button(a, true));
    harness.event(egui::Event::PointerMoved(b));
    harness.event(button(b, false));
    harness.run();
    match harness.state().intents.as_slice() {
        [Intent::Apply { label, .. }] => assert_eq!(label, "Rectángulo · 14 píxeles"),
        other => panic!("{other:?}"),
    }
}

/// Arrastra con el botón principal de un píxel a otro (eventos en orden).
fn drag(harness: &mut Harness<'static, State>, from: (i64, i64), to: (i64, i64)) {
    let vp = harness.state().shell.canvas.viewport.unwrap();
    let (a, b) = (
        vp.pixel_rect(from.0, from.1).center(),
        vp.pixel_rect(to.0, to.1).center(),
    );
    press(harness, a, true, egui::PointerButton::Primary);
    harness.event(egui::Event::PointerMoved(b));
    harness.run();
    press(harness, b, false, egui::PointerButton::Primary);
}

/// La mano mueve la vista y no toca el documento.
#[test]
fn the_hand_does_not_modify_the_document() {
    let mut harness = harness();
    let before = harness.state().vm.clone();
    harness.key_press(egui::Key::H);
    harness.run();
    let pan = harness.state().shell.canvas.pan;
    drag(&mut harness, (5, 5), (15, 9));
    assert!(harness.state().intents.is_empty());
    assert_eq!(harness.state().vm, before);
    assert_ne!(harness.state().shell.canvas.pan, pan, "la vista se movió");
}

/// Seleccionar, mover como un solo intent y Esc para quitar la selección.
#[test]
fn select_move_and_escape() {
    let mut harness = harness();
    harness.key_press(egui::Key::M);
    harness.run();
    drag(&mut harness, (10, 18), (21, 27));
    let sel = harness.state().shell.selection.expect("hay selección");
    assert_eq!((sel.x, sel.y, sel.w, sel.h), (10, 18, 12, 10));

    harness.key_press(egui::Key::V);
    harness.run();
    drag(&mut harness, (12, 20), (14, 19));
    match harness.state().intents.as_slice() {
        [Intent::Apply { label, command }] => {
            assert!(label.starts_with("Mover · "), "{label}");
            assert!(matches!(command, lume_domain::Command::Batch(_)));
        }
        other => panic!("{other:?}"),
    }
    let moved = harness.state().shell.selection.unwrap();
    assert_eq!(
        (moved.x, moved.y),
        (12, 17),
        "la selección acompaña al contenido"
    );

    // Esc durante un arrastre lo cancela; después, Esc quita la selección.
    let vp = harness.state().shell.canvas.viewport.unwrap();
    press(
        &mut harness,
        vp.pixel_rect(13, 18).center(),
        true,
        egui::PointerButton::Primary,
    );
    harness.key_press(egui::Key::Escape);
    harness.run();
    press(
        &mut harness,
        vp.pixel_rect(20, 20).center(),
        false,
        egui::PointerButton::Primary,
    );
    assert_eq!(
        harness.state().intents.len(),
        1,
        "el arrastre cancelado no mueve"
    );
    assert!(harness.state().shell.selection.is_some());
    harness.key_press(egui::Key::Escape);
    harness.run();
    assert!(harness.state().shell.selection.is_none());
}

/// Un documento con un batch de 22 comandos del agente y un trazo tuyo.
fn view_model_with_history() -> ViewModel {
    let mut doc = Document::new("historial", 32, 32).unwrap();
    let script: String = (0..22)
        .map(|i| format!("set_pixel {i} 0 #8b3fe0\n"))
        .collect();
    doc.run_batch(
        &script,
        Origin::Agent { run: 3 },
        &|p: &str| p.to_string(),
        &mut |_, _, _| Ok(()),
    )
    .unwrap();
    doc.begin(Origin::User, "Lápiz · 3 píxeles").unwrap();
    doc.apply(
        lume_domain::Command::FillRect {
            x: 0,
            y: 5,
            w: 3,
            h: 1,
            color: lume_domain::Rgba::rgb(0x1B, 0x10, 0x26),
        },
        Origin::User,
    )
    .unwrap();
    doc.commit();
    ViewModel::of(&doc, 2, None)
}

/// El historial muestra autor y número, y permite volver a una entrada.
#[test]
fn history_shows_entries_and_goes_back() {
    let mut harness = harness();
    harness.state_mut().vm = view_model_with_history();
    harness.run();
    // El historial está en su pestaña del panel derecho.
    assert!(harness.query_by_label("Lápiz · 3 píxeles — tú").is_none());
    harness
        .get_by_role_and_label(egui::Role::Tab, "Historial")
        .click();
    harness.run();
    harness.get_by_label("Lápiz · 3 píxeles — tú");
    harness.get_by_label("batch · 22 comandos — agente").click();
    harness.run();
    assert_eq!(harness.state().intents, vec![Intent::UndoSteps(1)]);
    // La entrada más reciente no hace nada (ya estás ahí).
    harness.get_by_label("Lápiz · 3 píxeles — tú").click();
    harness.run();
    assert_eq!(harness.state().intents.len(), 1);
}

/// La paleta sigue al lienzo y elegir una casilla cambia los colores.
#[test]
fn palette_follows_the_canvas_and_picks_colors() {
    let mut harness = harness();
    // view_model() tiene 4 colores.
    harness.get_by_label("4 colores");
    harness.get_by_label("#A86B3C").click();
    harness.run();
    assert_eq!(
        harness.state().shell.primary,
        lume_domain::Rgba::rgb(0xA8, 0x6B, 0x3C)
    );
    harness.get_by_label("#DCEEF6").click_secondary();
    harness.run();
    assert_eq!(
        harness.state().shell.secondary,
        lume_domain::Rgba::rgb(0xDC, 0xEE, 0xF6)
    );

    // Llega un ViewModel nuevo con un color más: aparece sin hacer nada.
    let mut doc = Document::new("pocion_morada", 32, 32).unwrap();
    for line in [
        "fill_rect 10 18 12 10 #8b3fe0",
        "fill_rect 13 3 6 3 #a86b3c",
        "line 12 6 19 6 #dceef6",
        "set_pixel 21 24 #5b2aa6",
        "set_pixel 0 0 #00ff00",
    ] {
        let _ = doc.handle_line(line, Origin::Cli);
    }
    harness.state_mut().vm = ViewModel::of(&doc, 5, None);
    harness.run();
    harness.get_by_label("5 colores");
    harness.get_by_label("#00FF00");
    assert!(
        harness.state().intents.is_empty(),
        "elegir colores no toca el documento"
    );
}

/// Un `ViewModel` cuyo último cambio vino de `origin`.
fn view_model_after(origin: Origin, line: &str) -> ViewModel {
    let mut doc = Document::new("pocion_morada", 32, 32).unwrap();
    let request = lume_domain::protocol::parse_request(line).unwrap();
    let lume_domain::Request::Apply(command) = request else {
        unreachable!()
    };
    let patch = doc.apply(command, origin).unwrap();
    let change = lume_app::Change {
        revision: 1,
        origin,
        label: line.to_string(),
        patch: std::sync::Arc::new(patch),
        lines: std::sync::Arc::new(vec![line.to_string()]),
    };
    ViewModel::of(&doc, 1, Some(change))
}

/// El último comando de la CLI o del agente se resalta con su etiqueta; los
/// trazos propios no; y se puede desactivar.
#[test]
fn the_last_command_is_highlighted() {
    let mut harness = harness();
    // De fábrica está apagado (se ve dibujar con el pincel); aquí se enciende.
    assert!(!harness.state().shell.show_commands);
    harness.state_mut().shell.show_commands = true;
    harness.state_mut().vm = view_model_after(Origin::Agent { run: 2 }, "set_pixel 21 24 #5b2aa6");
    harness.run();
    harness.get_by_label("agente · set_pixel 21 24");

    harness.state_mut().vm = view_model_after(Origin::Cli, "fill_rect 2 2 4 3 #ff0000");
    harness.run();
    harness.get_by_label("cli · fill_rect 2 2 4 3");
    assert!(harness.query_by_label("agente · set_pixel 21 24").is_none());

    // «Mostrar cada comando en el lienzo» apagado: sin resalte.
    harness.state_mut().shell.show_commands = false;
    harness.run();
    assert!(harness.query_by_label("cli · fill_rect 2 2 4 3").is_none());

    // Lo que dibujas tú no se resalta.
    harness.state_mut().shell.show_commands = true;
    harness.state_mut().vm = view_model_after(Origin::User, "set_pixel 1 1 #ffffff");
    harness.run();
    assert!(harness.query_by_label_contains("set_pixel 1 1").is_none());
}

/// Pasar el ratón por el lienzo no mueve nada de la barra de estado: las
/// coordenadas, el color bajo el cursor y el zoom tienen sitio fijo.
#[test]
fn the_status_bar_does_not_shift_under_the_pointer() {
    let mut harness = harness();
    let viewport = harness.state().shell.canvas.viewport.unwrap();
    harness.hover_at(viewport.pixel_rect(-3, -3).center());
    harness.run();
    let size = harness.get_by_label("32 × 32 px").rect();
    let zones = harness.state().shell.zones;
    // Coordenadas de una y dos cifras, color opaco (7) y con alfa (9).
    for (x, y) in [(0, 0), (21, 24), (5, 31), (31, 9), (12, 20)] {
        harness.hover_at(viewport.pixel_rect(x, y).center());
        harness.run();
        assert_eq!(
            harness.state().shell.canvas.hover,
            Some((x as u32, y as u32))
        );
        assert_eq!(
            harness.get_by_label("32 × 32 px").rect(),
            size,
            "sobre ({x}, {y})"
        );
        // Ni la barra ni lo de encima cambian de sitio (la barra crecía
        // 3 px con la muestra del color y empujaba el lienzo y el panel).
        assert_eq!(harness.state().shell.zones, zones, "sobre ({x}, {y})");
        assert_eq!(harness.state().shell.canvas.viewport, Some(viewport));
    }
}

/// Las casillas de los colores principal y secundario abren el selector de
/// color, que sigue abierto mientras se elige (un clic dentro no lo cierra).
#[test]
fn the_color_swatches_open_the_picker() {
    let mut harness = harness();
    // El selector de egui tiene un botón «📋»: en la ventana sale de las
    // fuentes del sistema (eframe las busca), que el arnés no carga.
    harness
        .ctx
        .set_missing_glyph_policy(egui::MissingGlyphPolicy::Tofu);
    for (name, primary) in [("Color principal", true), ("Color secundario", false)] {
        assert!(!egui::Popup::is_any_open(&harness.ctx), "{name}");
        harness.get_by_label(name).click();
        harness.run();
        assert!(egui::Popup::is_any_open(&harness.ctx), "{name}: abierto");

        // Rojo, verde y azul del selector.
        let fields: Vec<_> = harness
            .get_all_by_role(egui::Role::SpinButton)
            .map(|n| n.rect())
            .collect();
        assert!(fields.len() >= 3, "{name}: {}", fields.len());
        harness.hover_at(fields[0].center());
        harness.drag_at(fields[0].center());
        harness.run();
        harness.drop_at(fields[0].center());
        harness.run();
        assert!(
            egui::Popup::is_any_open(&harness.ctx),
            "{name}: sigue abierto tras un clic dentro"
        );
        harness
            .get_all_by_role(egui::Role::SpinButton)
            .next()
            .unwrap()
            .set_value_accesskit(17.0);
        harness.run();
        let shell = &harness.state().shell;
        let color = if primary {
            shell.primary
        } else {
            shell.secondary
        };
        assert_eq!(color.r, 17, "{name}");

        // Un clic fuera lo cierra.
        harness.hover_at(egui::pos2(700.0, 400.0));
        harness.drag_at(egui::pos2(700.0, 400.0));
        harness.run();
        harness.drop_at(egui::pos2(700.0, 400.0));
        harness.run();
        assert!(!egui::Popup::is_any_open(&harness.ctx), "{name}: cerrado");
    }
    // El «+» de la paleta usa el mismo selector.
    harness.get_by_label("Añadir color").click();
    harness.run();
    assert!(egui::Popup::is_any_open(&harness.ctx));
}

/// Un `ViewModel` tras un batch del agente con estas líneas.
fn view_model_after_batch(lines: &[&str]) -> ViewModel {
    let mut doc = Document::new("perro", 32, 32).unwrap();
    let before = doc.canvas().clone();
    doc.run_batch(
        &lines.join("\n"),
        Origin::Agent { run: 1 },
        &|p: &str| p.to_string(),
        &mut |_, _, _| Ok(()),
    )
    .unwrap();
    let change = lume_app::Change {
        revision: 9,
        origin: Origin::Agent { run: 1 },
        label: "batch".into(),
        patch: std::sync::Arc::new(lume_domain::Patch::between(&before, doc.canvas())),
        lines: std::sync::Arc::new(lines.iter().map(|s| s.to_string()).collect()),
    };
    ViewModel::of(&doc, 9, Some(change))
}

/// El dibujo del agente se ve hacer píxel a píxel (no de golpe) y la
/// pestaña Código va marcando la línea que se pinta; el selector de
/// velocidad pide guardar la elegida.
#[test]
fn the_agent_is_seen_drawing_and_the_code_tab_follows() {
    let mut harness = harness();
    harness
        .get_by_role_and_label(egui::Role::Tab, "Código")
        .click();
    harness.run();
    harness.state_mut().vm =
        view_model_after_batch(&["fill_rect 0 0 8 8 #ff0000", "set_pixel 31 31 #00ff00"]);
    harness.step();
    let painted = |h: &Harness<'static, State>| {
        h.state()
            .shell
            .canvas
            .brush
            .display()
            .map(|c| c.pixels().iter().filter(|p| p.a > 0).count())
    };
    let shell = &harness.state().shell;
    assert!(shell.canvas.brush.is_painting());
    let first = painted(&harness).unwrap();
    assert!(first > 0 && first < 65, "poco a poco: {first}");
    harness.get_by_label("línea 1: fill_rect 0 0 8 8 #ff0000 (pintando)");
    harness.get_by_label("línea 2: set_pixel 31 31 #00ff00 (pendiente)");
    harness.get_by_label("agente · 2 líneas · pintando…");

    for _ in 0..40 {
        if !harness.state().shell.canvas.brush.is_painting() {
            break;
        }
        harness.step();
    }
    assert!(!harness.state().shell.canvas.brush.is_painting());
    harness.step();
    harness.get_by_label("línea 2: set_pixel 31 31 #00ff00 (pintada)");
    harness.get_by_label("agente · 2 líneas · hecho");

    // Velocidad: «Al instante» se pide guardar.
    harness.get_by_label("Velocidad de dibujo").click();
    harness.run();
    harness.get_by_label("Al instante").click();
    harness.run();
    assert!(
        harness
            .state()
            .intents
            .contains(&Intent::DrawSpeed(lume_app::DrawSpeed::Instant))
    );
}
