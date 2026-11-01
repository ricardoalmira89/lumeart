//! Las formas que quedan bajo el lazo y bajo el cursor, con el SVG del
//! studio que genera `dibujo.js`.

use lume_app::ports::ShapeFinder;
use lume_domain::vec::studio::{Zone, simplify};
use lume_raster::ResvgRasterizer;

const DRAWING: &str = r##"canvas({ width: 400, height: 300, background: "#102030" })
layer("mar", () => {
  rect(0, 150, 400, 150, { fill: "#2f6f73" })
  circle(100, 200, 30, { fill: "#ffffff" })
})
layer("sol", () => {
  circle(300, 80, 40, { fill: "#e8803a" })
})
function flor(x, y) {
  circle(x, y, 10, { fill: "#ffcc00" })
}
flor(40, 40)
flor(80, 40)
"##;

fn studio() -> String {
    let res = lume_script::vec::render(DRAWING);
    assert!(res.errors().is_empty(), "{:?}", res.issues);
    res.svg
}

/// Un lazo alrededor del círculo blanco: él y el rectángulo del mar, con
/// sus líneas, y el fondo como forma «grande».
#[test]
fn the_lasso_finds_the_shapes_under_it_and_their_lines() {
    let svg = studio();
    let zone: Vec<[f64; 2]> = vec![[60.0, 160.0], [140.0, 160.0], [140.0, 240.0], [60.0, 240.0]];
    let hits = ResvgRasterizer
        .shapes_under(&svg, 400.0, 300.0, &zone)
        .unwrap();
    let lines: Vec<(u32, bool)> = hits.iter().map(|h| (h.line, h.big)).collect();
    assert!(lines.contains(&(3, false)), "{hits:?}");
    assert!(lines.contains(&(4, false)), "{hits:?}");
    assert!(lines.contains(&(1, true)), "el fondo: {hits:?}");
    assert!(!lines.iter().any(|(l, _)| *l == 7), "el sol está fuera");
    let circle = hits.iter().find(|h| h.line == 4).unwrap();
    // π·30² de 80×80: un 44 %, encima en todo lo que cubre.
    assert!((circle.share - 0.44).abs() < 0.06, "{circle:?}");
    assert!((circle.top - circle.share).abs() < 0.02, "{circle:?}");
    assert_eq!(circle.layer, "mar");
    assert_eq!(circle.tag, "circle");
    let sea = hits.iter().find(|h| h.line == 3).unwrap();
    assert!(
        sea.share > 0.98 && (sea.top - (1.0 - circle.share)).abs() < 0.06,
        "{sea:?}"
    );
    let zone = Zone {
        points: simplify(&zone, 1.0),
        hits,
    };
    assert_eq!(zone.summary(), "2 formas de la capa «mar»");
    assert_eq!(zone.lines(), [3, 4]);
}

/// Las formas que salen de una función se separan por la llamada.
#[test]
fn shapes_from_a_function_are_told_apart_by_their_call() {
    let svg = studio();
    let zone = vec![[20.0, 20.0], [100.0, 20.0], [100.0, 60.0], [20.0, 60.0]];
    let hits = ResvgRasterizer
        .shapes_under(&svg, 400.0, 300.0, &zone)
        .unwrap();
    let flowers: Vec<&Vec<u32>> = hits
        .iter()
        .filter(|h| h.line == 10)
        .map(|h| &h.chain)
        .collect();
    assert_eq!(flowers.len(), 2, "{hits:?}");
    assert!(
        flowers.contains(&&vec![10, 12]) && flowers.contains(&&vec![10, 13]),
        "{flowers:?}"
    );
}

/// Un clic: la forma de encima en ese punto, o el fondo si no hay otra.
#[test]
fn a_click_picks_the_shape_on_top() {
    let svg = studio();
    let at = |x, y| {
        ResvgRasterizer
            .shape_at(&svg, 400.0, 300.0, [x, y])
            .unwrap()
            .map(|h| (h.line, h.layer))
    };
    assert_eq!(at(100.0, 200.0), Some((4, "mar".to_string())));
    assert_eq!(at(300.0, 80.0), Some((7, "sol".to_string())));
    assert_eq!(at(390.0, 290.0), Some((3, "mar".to_string())));
    assert_eq!(at(200.0, 10.0), Some((1, String::new())), "el fondo");
    assert_eq!(at(500.0, 500.0), None);
}

/// Las formas semitransparentes (un halo con opacidad 0,18) también
/// cuentan, con su superficie de verdad.
#[test]
fn translucent_shapes_count_too() {
    const HALO: &str = r##"canvas({ width: 400, height: 300, background: "#102030" })
layer("sol", () => {
  circle(200, 150, 100, { fill: alpha("#ffffff", 0.18) })
  circle(200, 150, 40, { fill: "#e8803a" })
})
"##;
    let res = lume_script::vec::render(HALO);
    assert!(res.errors().is_empty(), "{:?}", res.issues);
    let zone = vec![
        [150.0, 100.0],
        [250.0, 100.0],
        [250.0, 200.0],
        [150.0, 200.0],
    ];
    let hits = ResvgRasterizer
        .shapes_under(&res.svg, 400.0, 300.0, &zone)
        .unwrap();
    let halo = hits.iter().find(|h| h.line == 3).expect("el halo");
    // Cubre toda la zona (el radio 100 la contiene).
    assert!(halo.share > 0.98, "{halo:?}");
    let sun = hits.iter().find(|h| h.line == 4).expect("el sol");
    // π·40² de 100×100: un 50 %.
    assert!((sun.share - 0.50).abs() < 0.06, "{sun:?}");
    let zone = Zone { points: zone, hits };
    assert_eq!(zone.lines(), [3, 4]);
    // Un clic en el borde del halo (fuera del sol) da el halo.
    let at = ResvgRasterizer
        .shape_at(&res.svg, 400.0, 300.0, [200.0, 70.0])
        .unwrap()
        .map(|h| h.line);
    assert_eq!(at, Some(3));
}
