//! Operaciones de archivo con el almacenamiento de verdad.

use std::sync::Arc;

use lume_app::{DocumentStore, Session};
use lume_domain::{Command, Document, Origin, Rgba};
use lume_store::{FsRecents, FsStore};
use lume_ui::{AppController, FakeDialogs, Intent};

struct Setup {
    runtime: tokio::runtime::Runtime,
    session: Session,
    controller: AppController,
}

fn setup(dir: &std::path::Path) -> Setup {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(
            Document::new("sin título", 16, 16).unwrap(),
            Arc::new(FsStore),
        )
    };
    let controller = AppController::new(
        session.handle.clone(),
        runtime.handle().clone(),
        Arc::new(FsStore),
        Some(Arc::new(FsRecents::new(dir.join("recents.json")))),
        Arc::new(FakeDialogs::default()),
    );
    Setup {
        runtime,
        session,
        controller,
    }
}

/// Guardar y reabrir conserva lienzo, receta, versiones y notas.
#[test]
fn save_and_reopen_keeps_everything() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = setup(dir.path());
    // Un documento con versiones y notas (como lo dejará el bucle del agente).
    let mut doc = Document::new("pocion", 16, 16).unwrap();
    doc.apply(
        Command::FillRect {
            x: 2,
            y: 2,
            w: 4,
            h: 4,
            color: Rgba::rgb(0x8b, 0x3f, 0xe0),
        },
        Origin::Agent { run: 1 },
    )
    .unwrap();
    doc.create_version(Origin::Agent { run: 1 }, "Vuelta 1");
    doc.apply(
        Command::SetPixel {
            x: 3,
            y: 3,
            color: Rgba::new(0xcf, 0xdd, 0xe3, 0xaa),
        },
        Origin::Agent { run: 1 },
    )
    .unwrap();
    doc.create_version(Origin::Agent { run: 1 }, "Vuelta 2");
    doc.notes = "Falta el brillo del cristal.\n".into();
    s.runtime
        .block_on(s.session.handle.replace_document(doc))
        .unwrap();
    // Y un trazo tuyo después.
    s.runtime
        .block_on(
            s.session
                .handle
                .line("line 0 15 15 0 white", Origin::User, None),
        )
        .unwrap();
    let path = dir.path().join("pocion.lume");
    let outcome = s.controller.handle(Intent::SaveTo(path.clone()));
    assert_eq!(outcome.error, None);
    assert!(!s.controller.is_dirty());
    let before = s.runtime.block_on(s.session.handle.snapshot()).unwrap();

    // Otra ventana (otra sesión) lo abre.
    let mut other = setup(dir.path());
    let outcome = other.controller.handle(Intent::OpenPath(path.clone()));
    assert_eq!(outcome.error, None);
    let after = other
        .runtime
        .block_on(other.session.handle.snapshot())
        .unwrap();
    assert_eq!(after.name, "pocion");
    assert_eq!(after.canvas(), before.canvas());
    assert_eq!(after.recipe(), before.recipe());
    assert_eq!(after.versions(), before.versions());
    assert_eq!(after.versions().len(), 2);
    assert_eq!(after.notes, "Falta el brillo del cristal.\n");
    assert_eq!(after.recipe().rebuild().unwrap(), *after.canvas());
    // Y queda en recientes.
    let recents = lume_app::RecentStore::list(&FsRecents::new(dir.path().join("recents.json")));
    assert_eq!(recents[0].path, path);
    for setup in [s, other] {
        setup.runtime.block_on(setup.session.close());
    }
}

/// Exportar PNG respeta la escala elegida.
#[test]
fn export_png_respects_the_scale() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = setup(dir.path());
    s.runtime
        .block_on(
            s.session
                .handle
                .line("set_pixel 15 15 #12345678", Origin::User, None),
        )
        .unwrap();
    for scale in [1u32, 4, 10] {
        let path = dir.path().join(format!("x{scale}.png"));
        let outcome = s.controller.handle(Intent::ExportPngTo {
            path: path.clone(),
            scale,
        });
        assert_eq!(outcome.error, None);
        let png = FsStore.read_png(&path).unwrap();
        assert_eq!((png.width(), png.height()), (16 * scale, 16 * scale));
        let last = i64::from(16 * scale - 1);
        assert_eq!(
            png.get(last, last).unwrap(),
            Rgba::new(0x12, 0x34, 0x56, 0x78)
        );
    }
    s.runtime.block_on(s.session.close());
}
