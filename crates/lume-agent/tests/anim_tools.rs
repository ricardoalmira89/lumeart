//! Herramientas de animación del agente con el motor de verdad: leer,
//! escribir (un paso de deshacer, con sus comprobaciones) y comprobar (con
//! la hoja de revisión como imagen).

use std::sync::Arc;

use lume_agent::SessionTools;
use lume_app::Session;
use lume_app::ports::{AgentTools, ContentPart};
use lume_app::testing::MemoryStore;
use lume_domain::anim::animation::{AnimEdit, Animation};
use lume_domain::{Document, Origin};
use lume_script::QuickJsAnim;

const SOURCE: &str = "config({ cell: [8, 8] });\npart(\"p\", () => { fill_rect(2, 2, 4, 4, \"#e8c070\"); });\nrest(() => { draw(\"p\"); });\naction(\"latir\", { frames: 2 }, (f) => { move(0, f, () => { draw(\"p\"); }); });\n";

#[test]
fn the_agent_reads_writes_and_checks_anim_js() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = {
        let _guard = rt.enter();
        Session::spawn(Document::new("gema", 8, 8).unwrap(), store.clone())
    };
    rt.block_on(
        session
            .handle
            .line("fill_rect 2 2 4 4 #e8c070", Origin::User, None),
    )
    .unwrap();
    rt.block_on(
        session
            .handle
            .edit_animation(AnimEdit::Set(Some(Animation::new(SOURCE))), Origin::User),
    )
    .unwrap();

    let plain = SessionTools::new(session.handle.clone(), store.clone(), 3);
    assert!(
        !plain.specs().iter().any(|s| s.name == "write_anim"),
        "sin motor, sin herramientas de animación"
    );
    let tools = SessionTools::new(session.handle.clone(), store.clone(), 3)
        .with_anim(Arc::new(QuickJsAnim));
    let names: Vec<String> = tools.specs().into_iter().map(|s| s.name).collect();
    assert!(
        names.ends_with(&[
            "read_anim".to_string(),
            "write_anim".into(),
            "check_anim".into()
        ]),
        "{names:?}"
    );

    let read = rt.block_on(tools.call("read_anim", "{}"));
    assert_eq!(read.content, [ContentPart::Text(SOURCE.into())]);

    let check = rt.block_on(tools.call("check_anim", "{}"));
    assert!(!check.is_error);
    let ContentPart::Text(text) = &check.content[0] else {
        panic!()
    };
    assert!(
        text.starts_with("Celda 8x8. Fila 1: reposo. Fila 2: latir (2 frames, 8 fps, bucle)."),
        "{text}"
    );
    assert!(
        matches!(check.content[1], ContentPart::Image { .. }),
        "la hoja de revisión"
    );

    // Un anim.js con un error: se guarda (se puede deshacer) y lo dice.
    let broken = SOURCE.replace("#e8c070", "nocolor");
    let input = serde_json::json!({ "source": broken }).to_string();
    let write = rt.block_on(tools.call("write_anim", &input));
    assert!(write.is_error);
    let ContentPart::Text(text) = &write.content[0] else {
        panic!()
    };
    assert!(text.starts_with("ok: anim.js guardado."), "{text}");
    assert!(text.contains("ERROR (línea"), "{text}");
    let vm = session.handle.view().borrow().clone();
    assert_eq!(vm.animation.as_ref().unwrap().source, broken);
    assert_eq!(vm.history.last().unwrap().origin, Origin::Agent { run: 3 });
    rt.block_on(session.handle.line("undo", Origin::User, None))
        .unwrap();
    assert_eq!(
        session
            .handle
            .view()
            .borrow()
            .animation
            .as_ref()
            .unwrap()
            .source,
        SOURCE
    );
}
