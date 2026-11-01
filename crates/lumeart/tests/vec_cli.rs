//! `lumeart vec`: el binario real sobre carpetas temporales. Las salidas
//! esperadas son las de `pxcli vec` con los mismos archivos (salvo donde se
//! dice).

use std::path::Path;
use std::process::Output;

/// El binario, sin VectorCraft aunque esté instalado y con su caché en la
/// carpeta del test.
fn lumeart(dir: &Path, args: &[&str]) -> Output {
    std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
        .args(args)
        .current_dir(dir)
        .env("LUMEART_VECTORCRAFT", "no")
        .env("XDG_CACHE_HOME", dir.join(".cache"))
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// Ancho y alto de un PNG.
fn png_size(path: &Path) -> (u32, u32) {
    let bytes = std::fs::read(path).unwrap();
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    (w, h)
}

fn drawing(dir: &Path, source: &str) {
    std::fs::write(dir.join("dibujo.js"), source).unwrap();
}

const ERRORS: &str =
    "Error: dibujo.js tiene 1 error(es): lo dibujado hasta el error sí se ha generado\n";

/// `vec check` dice cada error con su línea y termina con error, como
/// pxcli.
#[test]
fn check_reports_errors_with_their_line() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();

    drawing(
        dir,
        "canvas({ width: 400, height: 300, background: \"#f2ede4\" })\ncircle(200, 150, 80, { fill: \"#e8803a\" })\ncircel(1, 2, 3)\n",
    );
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        text(&out.stdout),
        "400x300 · 1 formas · SVG de 0 KB\nERROR (línea 3): circel no existe (¿una errata o una función que no está en la librería?)\n"
    );
    assert_eq!(text(&out.stderr), ERRORS);

    // Un argumento mal en una llamada de varias líneas dentro de una
    // función: la línea es la del `(` de la llamada, como en pxcli.
    drawing(
        dir,
        "canvas({ width: 400, height: 300, background: null })\nfunction caja(x, y) {\n  rect(x, y,\n    \"ancho\", 40,\n    { fill: \"red\" })\n}\ncircle(50, 50, 20, { fill: \"#2f6f73\" })\ncaja(10,\n  20)\n",
    );
    let out = lumeart(dir, &["vec", "check", "dibujo.js"]);
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(
        text(&out.stdout),
        "400x300 · 1 formas · SVG de 0 KB\nERROR (línea 3): rect: el ancho tiene que ser un número (es ancho)\n"
    );

    // Error de sintaxis: la misma línea que pxcli; el texto, el de QuickJS.
    drawing(
        dir,
        "canvas({ width: 400, height: 300 })\ncircle(1, 2, 3))\n",
    );
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(
        text(&out.stdout).starts_with("1024x1024 · 0 formas · SVG de 0 KB\nERROR (línea 2): "),
        "{}",
        text(&out.stdout)
    );

    // Sin formas: solo un aviso, y sale bien.
    drawing(
        dir,
        "canvas({ width: 120, height: 80, background: null })\n",
    );
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        text(&out.stdout),
        "120x80 · 0 formas · SVG de 0 KB\naviso: el dibujo está vacío: todavía no hay ninguna forma\n"
    );

    // Un bucle infinito se corta a los 4 s y señala el bucle (pxcli, la
    // línea que se ejecutaba al cortar).
    drawing(
        dir,
        "canvas({ width: 100, height: 100 })\nlet i = 0\nwhile (true) {\n  i++\n}\n",
    );
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(
        text(&out.stdout),
        "100x100 · 0 formas · SVG de 0 KB\nERROR (línea 3): se ha superado el tiempo límite (¿un bucle infinito?)\n"
    );

    let out = lumeart(dir, &["vec", "check", "otro.js"]);
    assert_eq!(
        text(&out.stderr),
        "Error: no se puede leer otro.js: open otro.js: no such file or directory\n"
    );
}

#[test]
fn init_svg_and_render_like_pxcli() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let out = lumeart(
        dir,
        &["vec", "init", "--size", "400x300", "-o", "arte/dibujo.js"],
    );
    assert_eq!(
        text(&out.stdout),
        "arte/dibujo.js creado: lienzo de 400x300.\n"
    );
    let out = lumeart(
        dir,
        &["vec", "init", "--size", "400x300", "-o", "arte/dibujo.js"],
    );
    assert_eq!(
        text(&out.stderr),
        "Error: arte/dibujo.js ya existe (usa --force para sobrescribirlo)\n"
    );
    let out = lumeart(dir, &["vec", "init", "--size", "8x8"]);
    assert_eq!(
        text(&out.stderr),
        "Error: tamaño no válido \"8x8\" (entre 16 y 4096)\n"
    );

    drawing(
        dir,
        "canvas({ width: 400, height: 300, background: null })\ncircle(200, 150, 80, { fill: \"#e8803a\" })\nrect(20, 20, 60, 40, {\n  fill: \"#2f6f73\"\n})\n",
    );
    let out = lumeart(dir, &["vec", "svg"]);
    assert_eq!(text(&out.stdout), "dibujo.svg guardado (2 formas, 0 KB)\n");
    let svg = std::fs::read_to_string(dir.join("dibujo.svg")).unwrap();
    assert!(svg.starts_with(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 400 300" width="400" height="300">"#
    ));
    assert!(svg.contains(r#"data-name="rect línea 3""#), "{svg}");
    assert!(svg.ends_with("</svg>\n"));

    let out = lumeart(dir, &["vec", "render", "--grid", "50"]);
    assert_eq!(
        text(&out.stdout),
        "revision.png guardado (dibujo entero, 2 formas, ancho 1024 px)\n"
    );
    assert_eq!(png_size(&dir.join("revision.png")), (1024, 768));
    let out = lumeart(
        dir,
        &[
            "vec",
            "render",
            "--crop",
            "100,50,200,100",
            "--width",
            "400",
            "--bg",
            "#ffffff",
            "-o",
            "zona.png",
        ],
    );
    assert_eq!(
        text(&out.stdout),
        "zona.png guardado (zona 100,50,200,100, 2 formas, ancho 400 px)\n"
    );
    assert_eq!(png_size(&dir.join("zona.png")), (400, 200));
    let out = lumeart(dir, &["vec", "render", "--crop", "1,2,3"]);
    assert_eq!(
        text(&out.stderr),
        "Error: --crop tiene que ser x,y,ancho,alto\n"
    );
}

#[test]
fn ref_and_compare_put_the_reference_on_the_canvas() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    drawing(
        dir,
        "canvas({ width: 300, height: 200, background: \"#f2ede4\" })\ncircle(150, 100, 60, { fill: \"#e8803a\" })\n",
    );
    let out = lumeart(dir, &["vec", "compare"]);
    assert_eq!(
        text(&out.stderr),
        "Error: no hay imagen de referencia (referencia-*.png) en esta carpeta: indica una con --ref\n"
    );
    std::fs::write(dir.join("referencia-1.txt"), "no soy una imagen").unwrap();
    let icon = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/brand/icon-256.png");
    std::fs::copy(&icon, dir.join("referencia-0.png")).unwrap();

    // La más nueva que sea una imagen: referencia-0.png.
    let out = lumeart(dir, &["vec", "ref", "--width", "600"]);
    assert_eq!(
        text(&out.stdout),
        "ref.png guardado: referencia-0.png encajada en el lienzo de 300x200 (las coordenadas son las de dibujo.js)\n"
    );
    assert_eq!(png_size(&dir.join("ref.png")), (600, 400));

    let out = lumeart(dir, &["vec", "compare", "--grid", "100"]);
    assert_eq!(
        text(&out.stdout),
        "comparacion.png guardado (izquierda el dibujo, derecha la referencia)\n"
    );
    // 2 × 300 + 10 de separación.
    assert_eq!(png_size(&dir.join("comparacion.png")), (610, 200));
    let out = lumeart(
        dir,
        &["vec", "compare", "--mode", "encima", "-o", "encima.png"],
    );
    assert_eq!(
        text(&out.stdout),
        "encima.png guardado (el dibujo al 55 % sobre la referencia)\n"
    );
    assert_eq!(png_size(&dir.join("encima.png")), (1024, 683));
    let out = lumeart(dir, &["vec", "compare", "--mode", "arriba"]);
    assert_eq!(
        text(&out.stderr),
        "Error: modo \"arriba\" no válido (lado o encima)\n"
    );
    let out = lumeart(dir, &["vec", "compare", "--ref", "referencia-1.txt"]);
    assert_eq!(
        text(&out.stderr),
        "Error: referencia-1.txt no es una imagen PNG, JPG, GIF o WebP\n"
    );
}

/// `vec poses` con el maniquí de `reference/runs/`: mismo tamaño y salida
/// que pxcli.
#[test]
fn poses_draw_each_action_like_pxcli() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    let maniqui = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../lume-script/tests/fixtures/vec/20261029-120000-vec-maniqui-de-prueba/dibujo.js");
    std::fs::copy(maniqui, dir.join("dibujo.js")).unwrap();
    let out = lumeart(dir, &["vec", "poses"]);
    assert_eq!(
        text(&out.stdout),
        "poses.png guardado: primera fila, la pose del dibujo; después una fila por acción\n"
    );
    assert_eq!(png_size(&dir.join("poses.png")), (2112, 1446));
    let out = lumeart(
        dir,
        &[
            "vec",
            "poses",
            "--action",
            "andar",
            "--frames",
            "4",
            "-o",
            "andar.png",
        ],
    );
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(png_size(&dir.join("andar.png")), (1005, 945));
    let out = lumeart(dir, &["vec", "poses", "--action", "volar"]);
    assert_eq!(
        text(&out.stderr),
        "Error: no hay ninguna acción llamada \"volar\"\n"
    );

    drawing(
        dir,
        "canvas({ width: 200, height: 100 })\ncircle(50, 50, 20, { fill: \"#e8803a\" })\n",
    );
    let out = lumeart(dir, &["vec", "poses"]);
    assert_eq!(
        text(&out.stderr),
        "Error: el dibujo no tiene esqueleto con piezas\n"
    );
}

#[test]
fn godot_needs_a_project_and_something_to_export() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("sin");
    std::fs::create_dir_all(&dir).unwrap();
    drawing(
        &dir,
        "canvas({ width: 200, height: 100, background: \"#102030\" })\ncircle(50, 50, 20, { fill: \"#e8803a\" })\n",
    );
    let out = lumeart(&dir, &["vec", "godot"]);
    let canonical = dir.canonicalize().unwrap();
    let message = text(&out.stderr);
    assert!(
        message
            == format!(
                "Error: {} no está dentro de un proyecto de Godot (no encuentro project.godot)\n",
                dir.display()
            )
            || message
                == format!(
                    "Error: {} no está dentro de un proyecto de Godot (no encuentro project.godot)\n",
                    canonical.display()
                ),
        "{message}"
    );

    std::fs::write(dir.join("project.godot"), "config_version=5\n").unwrap();
    drawing(
        &dir,
        "canvas({ width: 200, height: 100, background: \"#102030\" })\n",
    );
    let out = lumeart(&dir, &["vec", "godot"]);
    assert_eq!(
        text(&out.stderr),
        "Error: el dibujo no tiene capas que exportar: usa layer() para cada franja de profundidad\n"
    );

    // Una escena sencilla: el fondo liso y una capa con su textura.
    drawing(
        &dir,
        "canvas({ width: 200, height: 100, background: \"#102030\" })\nlayer(\"cielo\", { parallax: [0.5, 1] }, () => {\n  circle(50, 50, 20, { fill: \"#e8803a\" })\n})\n",
    );
    let out = lumeart(&dir, &["vec", "godot", "-o", "nivel", "--name", "Mi Nivel"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout)
            .ends_with("mi_nivel.tscn\n1 texturas en texturas/ · 1 capas · 0 elementos\n"),
        "{}",
        text(&out.stdout)
    );
    let tscn = std::fs::read_to_string(dir.join("nivel/mi_nivel.tscn")).unwrap();
    assert!(tscn.starts_with("[gd_scene load_steps=2 format=3]\n\n[ext_resource type=\"Texture2D\" path=\"res://nivel/texturas/cielo__cielo.png\" id=\"1_tex\"]\n"), "{tscn}");
    assert!(
        tscn.contains("[node name=\"Mi_nivel\" type=\"Node2D\"]"),
        "{tscn}"
    );
    assert!(
        tscn.contains("color = Color(0.06275, 0.12549, 0.18824, 1)"),
        "{tscn}"
    );
    assert!(
        tscn.contains("scroll_scale = Vector2(0.5, 1)\nmetadata/capa = \"cielo\""),
        "{tscn}"
    );
    // El círculo con su margen: de 30-8 a 70+8.
    assert!(tscn.contains("position = Vector2(22, 22)"), "{tscn}");
    assert_eq!(
        png_size(&dir.join("nivel/texturas/cielo__cielo.png")),
        (56, 56)
    );
}

/// Una referencia de 120 × 80: rojo a la izquierda, azul a la derecha y una
/// franja oscura abajo.
fn reference_png(path: &Path) {
    let (w, h) = (120u32, 80u32);
    let mut rgba = Vec::new();
    for y in 0..h {
        for x in 0..w {
            let px = if y >= 60 {
                [20, 20, 20, 255]
            } else if x < 60 {
                [220, 30, 40, 255]
            } else {
                [30, 60, 200, 255]
            };
            rgba.extend_from_slice(&px);
        }
    }
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&rgba).unwrap();
}

/// `vec calco` lista los colores del calco y lo guarda sobre blanco;
/// `trace()` en `dibujo.js` calca con la paleta pedida.
#[test]
fn calco_traces_the_reference_with_the_requested_colours() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    drawing(
        dir,
        "canvas({ width: 240, height: 160, background: null })\n",
    );
    let out = lumeart(dir, &["vec", "calco"]);
    assert_eq!(
        text(&out.stderr),
        "Error: no hay imagen de referencia (referencia-*.png) en esta carpeta: indica una con --ref\n"
    );
    reference_png(&dir.join("referencia-1.png"));
    let out = lumeart(dir, &["vec", "calco", "--colors", "3", "--width", "480"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines[0].starts_with("Colores del calco ("), "{stdout}");
    assert!(
        lines[0].ends_with(" trazados), de más a menos usado:"),
        "{stdout}"
    );
    // Tres colores, cada uno «  #rrggbb      N trazados».
    assert_eq!(lines.len(), 5, "{stdout}");
    for line in &lines[1..4] {
        assert!(
            line.starts_with("  #") && line.ends_with(" trazados"),
            "{line}"
        );
    }
    assert_eq!(
        lines[4],
        "calco.png guardado: el calco de referencia-1.png solo, en las coordenadas de dibujo.js"
    );
    assert_eq!(png_size(&dir.join("calco.png")), (480, 320));
    let out = lumeart(
        dir,
        &["vec", "calco", "--mode", "lines", "-o", "lineas.png"],
    );
    assert_eq!(
        text(&out.stdout),
        "lineas.png guardado: el calco de referencia-1.png solo, en las coordenadas de dibujo.js\n"
    );

    // trace() con la paleta del dibujo: cada trazo, de un color de ella.
    drawing(
        dir,
        "canvas({ width: 240, height: 160, background: null })\ntrace({ colors: 3, noise: 2, palette: [\"#ff0000\", \"#0000ff\", \"#000000\", \"#00ff00\"] })\n",
    );
    let out = lumeart(dir, &["vec", "svg"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stdout));
    let svg = std::fs::read_to_string(dir.join("dibujo.svg")).unwrap();
    let fills: std::collections::BTreeSet<&str> = svg
        .split("fill=\"")
        .skip(1)
        .map(|rest| &rest[..7])
        .collect();
    assert_eq!(
        fills.into_iter().collect::<Vec<_>>(),
        ["#000000", "#0000ff", "#ff0000"]
    );

    // Una referencia que no existe, con la línea de trace().
    drawing(
        dir,
        "canvas({ width: 240, height: 160 })\ncircle(10, 10, 5)\ntrace({ ref: \"otra.png\" })\n",
    );
    let out = lumeart(dir, &["vec", "check"]);
    assert_eq!(
        text(&out.stdout),
        "240x160 · 1 formas · SVG de 0 KB\nERROR (línea 3): trace: otra.png no es una imagen PNG, JPG, GIF o WebP\n"
    );
}

/// `vec export`: PDF y `.ai` (un PDF) con una capa por grupo, SVG tal cual;
/// otros formatos, error; `.vectorcraft` y `.eps` piden VectorCraft.
#[test]
fn export_writes_pdf_ai_and_svg() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path();
    drawing(
        dir,
        "canvas({ width: 200, height: 100, background: \"#102030\" })\nlayer(\"cielo\", {}, () => {\n  circle(50, 50, 20, { fill: \"#e8803a\" })\n})\n",
    );
    for file in ["dibujo.pdf", "dibujo.ai"] {
        let out = lumeart(dir, &["vec", "export", "-o", file]);
        assert_eq!(text(&out.stdout), format!("{file} guardado (1 formas)\n"));
        let bytes = std::fs::read(dir.join(file)).unwrap();
        assert!(bytes.starts_with(b"%PDF-"), "{file}");
        let doc = String::from_utf8_lossy(&bytes);
        assert!(doc.contains("/OCProperties"), "{file}");
        assert!(
            doc.contains("/Name (Lienzo)") && doc.contains("/Name (cielo)"),
            "{file}"
        );
    }
    let out = lumeart(dir, &["vec", "export", "-o", "dibujo.svg"]);
    assert_eq!(text(&out.stdout), "dibujo.svg guardado (1 formas)\n");
    assert!(
        std::fs::read_to_string(dir.join("dibujo.svg"))
            .unwrap()
            .starts_with("<svg ")
    );
    let out = lumeart(dir, &["vec", "export", "-o", "dibujo.png"]);
    assert_eq!(
        text(&out.stderr),
        "Error: formato \".png\" no soportado: usa .vectorcraft, .ai, .pdf, .eps o .svg\n"
    );
    let out = lumeart(dir, &["vec", "export"]);
    assert_eq!(
        text(&out.stderr),
        "Error: hace falta VectorCraft: instala vectorcraft-cli (viene con VectorCraft) o indica su ruta en LUMEART_VECTORCRAFT\n"
    );
    drawing(
        dir,
        "canvas({ width: 200, height: 100, background: null })\n",
    );
    // Sin formas sigue habiendo SVG: se exporta y avisa, como pxcli.
    let out = lumeart(dir, &["vec", "export", "-o", "vacio.pdf"]);
    assert_eq!(
        text(&out.stdout),
        "vacio.pdf guardado (0 formas)\naviso: el dibujo está vacío: todavía no hay ninguna forma\n"
    );
}
