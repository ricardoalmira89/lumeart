//! Los tests de `internal/anim/anim_test.go` de pxcli, portados: receta,
//! esqueleto, piezas y movimientos, comprobaciones, hojas, líneas de los
//! errores, estado entre fotogramas, tiempo límite y `rand()`.

use std::sync::Arc;

use lume_app::ports::BaseLoader;
use lume_domain::anim::check::{check, place_base};
use lume_domain::anim::recipe::{Cmd, clean_recipe, paint, parse_recipe, recipe_from_image};
use lume_domain::anim::sheet::{review, sheet, strip};
use lume_domain::anim::skeleton::{SkeletonOptions, origin, skeleton};
use lume_domain::anim::{ActionSpec, AnimResult, Image, Level, same_image};
use lume_script::anim::render as engine_render;

/// Un reloj de arena de 12×12, como receta de pxcli.
const HOURGLASS: &[&str] = &[
    "fill_rect 2 0 8 1 #7a4a2a",
    "fill_rect 2 11 8 1 #7a4a2a",
    "fill_rect 3 1 6 10 #cfe8ff",
    "set_pixel 0 0 #ff0000", // se tapa abajo: la limpieza lo quita
    "fill_rect 0 0 1 1 transparent",
    "fill_rect 4 2 4 3 #e8c070",
    "line 6 5 6 9 #e8c070",
];

fn base_image() -> (Image, Vec<Cmd>) {
    let cmds = parse_recipe(HOURGLASS).unwrap();
    let mut img = Image::new(12, 12);
    paint(&mut img, &cmds, None);
    (img, cmds)
}

/// `render` + `Check`, con la base para cualquier nombre (o sin base).
fn render(src: &str, base: Option<Image>) -> AnimResult {
    let loader: BaseLoader =
        Arc::new(move |_name: &str| base.clone().ok_or_else(|| "no existe".to_string()));
    let mut res = engine_render(src, loader);
    check(&mut res);
    res
}

fn has_issue(res: &AnimResult, level: Level, text: &str) -> bool {
    res.issues
        .iter()
        .any(|i| i.level == level && i.msg.contains(text))
}

#[test]
fn clean_recipe_keeps_the_image() {
    let (base, cmds) = base_image();
    let clean = clean_recipe(&cmds, 12, 12);
    assert!(
        clean.len() < cmds.len(),
        "el píxel tapado debería caer: {} de {}",
        clean.len(),
        cmds.len()
    );
    let mut img = Image::new(12, 12);
    paint(&mut img, &clean, None);
    assert!(same_image(&img, &base), "limpiar cambió la imagen");
    assert!(parse_recipe(&["undo"]).is_err(), "undo no es de dibujo");
}

#[test]
fn recipe_from_image_rebuilds_exactly() {
    let (base, _) = base_image();
    let mut img = Image::new(12, 12);
    paint(&mut img, &recipe_from_image(&base), None);
    assert!(same_image(&img, &base));
}

#[test]
fn skeleton_rest_matches_base() {
    let (base, cmds) = base_image();
    let src = skeleton(&SkeletonOptions {
        name: "reloj".into(),
        base: "base.png".into(),
        cell: (16, 16),
        origin: origin((12, 12), (16, 16), "bottom"),
        recipe: clean_recipe(&cmds, 12, 12),
        from_pixels: false,
        actions: vec![ActionSpec {
            name: "caer".into(),
            frames: 4,
            fps: 8,
            looping: true,
        }],
    });
    let res = render(&src, Some(base));
    assert!(res.errors().is_empty(), "{:?}\n{src}", res.errors());
    assert_eq!(res.cell, (16, 16));
    assert_eq!(res.origin, (2, 4));
    assert!(
        has_issue(&res, Level::Warning, "no se mueve"),
        "una acción del esqueleto aún no se mueve: {:?}",
        res.issues
    );
}

#[test]
fn parts_moves_and_checks() {
    let (base, _) = base_image();
    let src = r##"
config({ base: "base.png", cell: [12, 12] });
part("marco", () => { fill_rect(2, 0, 8, 1, "#7a4a2a"); fill_rect(2, 11, 8, 1, "#7a4a2a"); fill_rect(3, 1, 6, 10, "#cfe8ff"); });
part("arena", ({ nivel = 0 }) => { fill_rect(4, 2 + nivel, 4, 3 - nivel, "#e8c070"); line(6, 5, 6, 9, "#e8c070"); });
rest(() => { draw("marco"); draw("arena"); });
action("caer", { frames: 4, fps: 8 }, (f) => { draw("marco"); draw("arena", { nivel: Math.min(f, 3) }); });
action("salto", { frames: 3, loop: false }, (f, { t }) => { move(0, -Math.round(t * 4), () => { draw("marco"); }); });
action("espejo", { frames: 2 }, (f) => { flip_x(() => { set_pixel(0, 0, "#7a4a2a"); }); set_pixel(f, 5, "#00ff00"); });
"##;
    let res = render(src, Some(base.clone()));
    assert!(res.errors().is_empty(), "{:?}", res.errors());
    let rest = res.rest.as_ref().expect("reposo");
    assert!(
        same_image(rest, &place_base(&res, &base)),
        "el reposo debería ser la base"
    );
    let caer = &res.actions[0];
    assert!(
        !same_image(&caer.images[0], &caer.images[2]),
        "el nivel de la arena cambia los fotogramas"
    );
    assert!(
        has_issue(&res, Level::Warning, "se salen de la celda"),
        "el salto saca el dibujo de la celda: {:?}",
        res.issues
    );
    assert!(
        has_issue(&res, Level::Warning, "#00ff00"),
        "el verde no está en la base: {:?}",
        res.issues
    );
    let espejo = &res.actions[2].images[0];
    assert!(
        espejo.get(11, 0).a > 0,
        "flip_x lleva x=0 a x=11 en una base de 12"
    );
    let s = sheet(&res, 1);
    assert_eq!(s.size(), (4 * 12, 3 * 12));
    assert_eq!(strip(caer, res.cell, 2).width(), 4 * 12 * 2);
    assert!(
        review(&res, 4).height() > 4 * 12 * 4,
        "la hoja de revisión añade el reposo y huecos"
    );
}

#[test]
fn rest_mismatch_is_an_error() {
    let (base, _) = base_image();
    let res = render(
        r##"config({ base: "base.png" }); rest(() => { fill_rect(2, 0, 8, 1, "#7a4a2a"); }); action("a", { frames: 1 }, () => { base(); });"##,
        Some(base),
    );
    assert!(
        has_issue(&res, Level::Error, "no coincide con la base"),
        "{:?}",
        res.issues
    );
}

#[test]
fn errors_carry_line_numbers() {
    let res = render(
        "config({ cell: [8, 8] });\naction(\"a\", { frames: 2 }, (f) => {\n  fill_rect(0, 0, 2, 2, \"nocolor\");\n});\n",
        None,
    );
    let errs = res.errors();
    assert!(
        !errs.is_empty() && errs[0].line == 3 && errs[0].msg.contains("color no válido"),
        "un error de color en la línea 3: {errs:?}"
    );
    let res = render(
        "config({ cell: [8, 8] });\naction(\"a\", {}, (f) => {\n  fill_rect(0, 0,, 2);\n});\n",
        None,
    );
    let errs = res.errors();
    assert!(
        !errs.is_empty() && errs[0].line == 3,
        "un error de sintaxis en la línea 3: {errs:?}"
    );
}

#[test]
fn state_between_frames_and_timeouts() {
    let res = render(
        r##"config({ cell: [8, 8] }); let x = 0; action("a", { frames: 4 }, (f) => { x++; set_pixel(x % 8, 0, "#fff"); });"##,
        None,
    );
    assert!(
        has_issue(&res, Level::Warning, "no guardes estado"),
        "{:?}",
        res.issues
    );
    let res = render(
        r##"config({ cell: [8, 8] }); action("a", { frames: 1 }, () => { while (true) {} });"##,
        None,
    );
    assert!(
        has_issue(&res, Level::Error, "tiempo límite"),
        "{:?}",
        res.issues
    );
    let src = r##"config({ cell: [8, 8] }); action("a", { frames: 3 }, (f) => { set_pixel(Math.floor(rand() * 8), f, "#fff"); });"##;
    let (a, b) = (render(src, None), render(src, None));
    for f in 0..3 {
        assert!(
            same_image(&a.actions[0].images[f], &b.actions[0].images[f]),
            "rand() se repite igual en el mismo fotograma"
        );
    }
}

/// Un bucle infinito se corta por tiempo con la línea donde estaba.
#[test]
fn an_endless_loop_is_cut_with_its_line() {
    let res = render(
        "config({ cell: [8, 8] });\nlet k = 0;\naction(\"gira\", { frames: 2 }, (f) => {\n  set_pixel(0, 0, \"#fff\");\n  while (true) { k++; }\n});\n",
        None,
    );
    let errs = res.errors();
    assert_eq!(errs.len(), 2, "uno por fotograma: {errs:?}");
    assert_eq!(errs[0].line, 5);
    assert_eq!(errs[0].place, "gira f0");
    assert_eq!(
        errs[0].msg,
        "se ha superado el tiempo límite (¿un bucle infinito?)"
    );
    // Al cargar también: 3 s como mucho, con su línea.
    let started = std::time::Instant::now();
    let res = render("let a = 1;\n\nfor (;;) {}\n", None);
    assert!(started.elapsed() < std::time::Duration::from_secs(6));
    let errs = res.errors();
    assert_eq!(errs[0].line, 3, "{errs:?}");
    assert!(errs[0].msg.contains("tiempo límite"));
}

/// Sin archivos ni red ni reloj: no hay `require`, `fetch` ni `Date`.
#[test]
fn the_sandbox_has_no_files_network_or_clock() {
    for (src, missing) in [
        ("require('fs')", "require"),
        ("fetch('http://x')", "fetch"),
        ("new Date()", "Date"),
    ] {
        let script = format!(
            "config({{ cell: [4, 4] }});\naction(\"a\", {{ frames: 1 }}, () => {{\n  {src};\n}});\n"
        );
        let res = render(&script, None);
        let errs = res.errors();
        assert_eq!(errs.len(), 1, "{missing}: {errs:?}");
        assert_eq!(errs[0].line, 3, "{missing}: {errs:?}");
    }
    // Math.random es rand(): repetible.
    let src = r##"config({ cell: [8, 8] }); action("a", { frames: 1 }, () => { set_pixel(Math.floor(Math.random() * 8), 0, "#fff"); });"##;
    assert_eq!(
        render(src, None).actions[0].images,
        render(src, None).actions[0].images
    );
}
