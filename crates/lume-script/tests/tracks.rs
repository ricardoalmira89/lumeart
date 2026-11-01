//! Las pistas de la línea de tiempo: una por pieza que usa la acción, con
//! rombo en el primer fotograma y donde la pieza cambia (otros parámetros u
//! otra posición).

use std::sync::Arc;

use lume_app::ports::BaseLoader;
use lume_domain::anim::tracks;
use lume_script::anim::render;

#[test]
fn diamonds_mark_where_each_part_changes() {
    let src = r##"
config({ cell: [12, 12] });
part("marco", () => { fill_rect(2, 0, 8, 1, "#7a4a2a"); });
part("arena", ({ nivel = 0 }) => { fill_rect(4, 2 + nivel, 4, 3 - nivel, "#e8c070"); });
part("chispa", () => { set_pixel(0, 0, "#ffffff"); });
part("sin_usar", () => { set_pixel(1, 1, "#ffffff"); });
action("caer", { frames: 5 }, (f) => {
  draw("marco");
  draw("arena", { nivel: Math.min(f, 2) });
  move(0, f == 4 ? 1 : 0, () => { if (f % 2 == 0) { draw("chispa"); } });
});
"##;
    let loader: BaseLoader = Arc::new(|_: &str| Err("sin base".to_string()));
    let res = render(src, loader);
    assert!(res.errors().is_empty(), "{:?}", res.errors());
    assert_eq!(res.parts, ["marco", "arena", "chispa", "sin_usar"]);
    let t = tracks(&res, &res.actions[0]);
    let names: Vec<&str> = t.iter().map(|t| t.part.as_str()).collect();
    assert_eq!(
        names,
        ["marco", "arena", "chispa"],
        "solo las que usa la acción"
    );
    // El marco no cambia: solo el primer fotograma es clave.
    assert_eq!(t[0].keys, [true, false, false, false, false]);
    // La arena cambia de nivel en 1 y 2; luego se queda en 2.
    assert_eq!(t[1].keys, [true, true, true, false, false]);
    // La chispa se dibuja en 0, 2 y 4 (en 4, movida): clave al reaparecer
    // y al moverse.
    assert_eq!(t[2].drawn, [true, false, true, false, true]);
    assert_eq!(t[2].keys, [true, false, true, false, true]);
}
