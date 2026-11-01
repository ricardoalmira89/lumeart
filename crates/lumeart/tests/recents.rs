//! Los recientes se leen del almacenamiento de verdad (`recents.json` +
//! `.lume`) y abren el documento en la sesión.

use std::sync::Arc;

use lume_app::{DocumentStore, RecentStore, Session};
use lume_domain::{Command, Document, Origin, Rgba};
use lume_store::{FsRecents, FsStore};
use lume_ui::{AppController, Intent, Screen};

#[test]
fn recents_are_read_from_storage_and_open_the_document() {
    let dir = tempfile::tempdir().unwrap();
    let mut doc = Document::new("seta_roja", 24, 24).unwrap();
    doc.apply(
        Command::FillRect {
            x: 2,
            y: 2,
            w: 5,
            h: 3,
            color: Rgba::rgb(0xe0, 0x30, 0x30),
        },
        Origin::User,
    )
    .unwrap();
    let path = dir.path().join("seta_roja.lume");
    FsStore.save_document(&doc, &path).unwrap();
    let recents = Arc::new(FsRecents::new(dir.path().join("datos/recents.json")));
    recents.touch(&path, 1_000).unwrap();

    // La pantalla de inicio lee esto.
    let list = recents.list();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "seta_roja");
    assert_eq!(list[0].thumbnail, *doc.canvas());

    // Y al pulsarlo se abre en la sesión.
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let session = {
        let _guard = runtime.enter();
        Session::spawn(Document::new("inicio", 32, 32).unwrap(), Arc::new(FsStore))
    };
    let mut controller = AppController::new(
        session.handle.clone(),
        runtime.handle().clone(),
        Arc::new(FsStore),
        Some(recents.clone()),
        Arc::new(lume_ui::FakeDialogs::default()),
    );
    let outcome = controller.handle(Intent::OpenRecent(list[0].path.clone()));
    assert_eq!(outcome.screen, Some(Screen::Studio));
    let vm = session.handle.view().borrow().clone();
    assert_eq!(vm.name, "seta_roja");
    assert_eq!(*vm.canvas, *doc.canvas());
    // Queda el primero de la lista, con la hora nueva.
    assert!(recents.list()[0].opened_at > 1_000);
    runtime.block_on(session.close());
}
