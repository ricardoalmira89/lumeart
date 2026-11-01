//! Tests de la `Session` (actor): orden, publicación y cierre.

use std::path::PathBuf;
use std::sync::Arc;

use lume_app::testing::MemoryStore;
use lume_app::{AppError, Session, SessionEvent};
use lume_domain::{Command, Document, Origin, Rgba};

fn spawn(w: i64, h: i64) -> (Session, Arc<MemoryStore>) {
    let store = Arc::new(MemoryStore::new());
    let session = Session::spawn(Document::new("test", w, h).unwrap(), store.clone());
    (session, store)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn hundred_parallel_commands_apply_in_arrival_order() {
    let (session, _) = spawn(10, 10);
    let mut tasks = Vec::new();
    for i in 0..100u8 {
        let handle = session.handle.clone();
        tasks.push(tokio::spawn(async move {
            let line = format!("set_pixel {} {} #{:02x}0000ff", i % 10, i / 10, i);
            let result = handle.line(line.clone(), Origin::Cli, None).await.unwrap();
            (line, result)
        }));
    }
    let mut results = Vec::new();
    for t in tasks {
        results.push(t.await.unwrap());
    }

    // Todos se aplicaron, cada uno con su propia revisión (1..=100).
    let mut revisions: Vec<u64> = results.iter().map(|(_, r)| r.revision).collect();
    assert!(results.iter().all(|(_, r)| r.response.is_ok()));
    revisions.sort();
    assert_eq!(revisions, (1..=100).collect::<Vec<_>>());

    // La receta está en orden de llegada: el comando con revisión N es el N.º.
    let doc = session.close().await;
    let recipe = doc.recipe().commands();
    assert_eq!(recipe.len(), 100);
    for (line, result) in &results {
        let in_recipe = recipe[result.revision as usize - 1].to_string();
        let expected = lume_domain::protocol::parse_request(line)
            .unwrap()
            .to_string();
        assert_eq!(in_recipe, expected);
    }
    for i in 0..100u8 {
        let c = doc
            .canvas()
            .get(i64::from(i % 10), i64::from(i / 10))
            .unwrap();
        assert_eq!(c, Rgba::rgb(i, 0, 0));
    }
}

#[tokio::test]
async fn each_change_publishes_a_view_model_and_an_event_with_the_patch() {
    let (session, _) = spawn(4, 4);
    let h = &session.handle;
    let mut events = h.subscribe();
    let mut view = h.view();
    assert_eq!(view.borrow().revision, 0);

    h.line("set_pixel 1 2 red", Origin::Cli, None)
        .await
        .unwrap();
    assert!(view.has_changed().unwrap());
    let vm = view.borrow_and_update().clone();
    assert_eq!(vm.revision, 1);
    assert_eq!(vm.canvas.get(1, 2).unwrap(), Rgba::rgb(255, 0, 0));
    assert_eq!(vm.history.len(), 1);
    assert!(vm.can_undo);
    let change = vm.last_change.clone().unwrap();
    assert_eq!(change.label, "set_pixel 1 2 red");

    match events.recv().await.unwrap() {
        SessionEvent::Changed(c) => {
            assert_eq!(c.revision, 1);
            assert_eq!(c.origin, Origin::Cli);
            let cells: Vec<_> = c.patch.cells().map(|c| (c.x, c.y, c.after)).collect();
            assert_eq!(cells, vec![(1, 2, Rgba::rgb(255, 0, 0))]);
        }
        other => panic!("{other:?}"),
    }
    match events.recv().await.unwrap() {
        SessionEvent::Command { line, response, .. } => {
            assert_eq!(
                (line.as_str(), response.as_str()),
                ("set_pixel 1 2 red", "ok")
            );
        }
        other => panic!("{other:?}"),
    }

    // Leer o fallar no cambia nada: sin ViewModel nuevo ni evento Changed.
    let r = h.line("get_pixel 1 2", Origin::Cli, None).await.unwrap();
    assert_eq!(r.response.to_string(), "ok #ff0000ff");
    let r = h
        .line("set_pixel 9 9 red", Origin::Cli, None)
        .await
        .unwrap();
    assert_eq!(
        r.response.to_string(),
        "err out_of_bounds pixel (9,9) outside canvas"
    );
    assert!(!view.has_changed().unwrap());
    for _ in 0..2 {
        assert!(matches!(
            events.recv().await.unwrap(),
            SessionEvent::Command { .. }
        ));
    }

    // Deshacer también es un cambio, con su patch.
    h.line("undo", Origin::User, None).await.unwrap();
    let vm = view.borrow_and_update().clone();
    assert_eq!(vm.revision, 2);
    assert_eq!(vm.canvas.get(1, 2).unwrap(), Rgba::TRANSPARENT);
    match events.recv().await.unwrap() {
        SessionEvent::Changed(c) => assert_eq!(c.patch.cell_count(), 1),
        other => panic!("{other:?}"),
    }
    session.close().await;
}

#[tokio::test]
async fn apply_and_batch_publish_one_change_each() {
    let (session, _) = spawn(4, 4);
    let h = &session.handle;
    let patch = h
        .apply(
            Command::FillRect {
                x: 0,
                y: 0,
                w: 2,
                h: 2,
                color: Rgba::rgb(0, 0, 255),
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(patch.cell_count(), 4);
    let report = h
        .batch(
            "set_pixel 3 3 red; set_pixel 9 9 red; line 0 3 3 0 white",
            Origin::Agent { run: 1 },
            None,
        )
        .await
        .unwrap();
    assert_eq!(report.summary(), "batch: 2 ok, 1 failed");
    let vm = h.view().borrow().clone();
    assert_eq!(vm.revision, 2);
    assert_eq!(vm.history.last().unwrap().label, "batch");
    assert_eq!(vm.history.last().unwrap().commands, 2);
    let err = h
        .apply(
            Command::SetPixel {
                x: -1,
                y: 0,
                color: Rgba::TRANSPARENT,
            },
            Origin::User,
        )
        .await;
    assert!(matches!(err, Err(AppError::Domain(e)) if e.message == "pixel (-1,0) outside canvas"));
    session.close().await;
}

/// El cambio de un `batch` lleva sus líneas (la ventana lo repite paso a
/// paso); el de una línea suelta, esa línea; el de la ventana, ninguna.
#[tokio::test]
async fn changes_carry_the_lines_that_made_them() {
    let (session, _) = spawn(4, 4);
    let h = &session.handle;
    h.batch(
        "set_pixel 3 3 red\n# nota\nline 0 3 3 0 white; lumeart set_pixel 0 0 blue",
        Origin::Agent { run: 1 },
        None,
    )
    .await
    .unwrap();
    let change = h.view().borrow().last_change.clone().unwrap();
    assert_eq!(
        change.lines.as_slice(),
        [
            "set_pixel 3 3 red",
            "line 0 3 3 0 white",
            "lumeart set_pixel 0 0 blue"
        ]
    );
    h.line("fill_rect 0 0 2 2 #00ff00", Origin::Cli, None)
        .await
        .unwrap();
    let change = h.view().borrow().last_change.clone().unwrap();
    assert_eq!(change.lines.as_slice(), ["fill_rect 0 0 2 2 #00ff00"]);
    h.apply(
        Command::SetPixel {
            x: 1,
            y: 1,
            color: Rgba::rgb(9, 9, 9),
        },
        Origin::User,
    )
    .await
    .unwrap();
    let change = h.view().borrow().last_change.clone().unwrap();
    assert!(change.lines.is_empty());
    session.close().await;
}

#[tokio::test]
async fn export_goes_through_the_store_with_the_client_folder() {
    let (session, store) = spawn(2, 2);
    let h = &session.handle;
    h.line("set_pixel 0 0 red", Origin::Cli, None)
        .await
        .unwrap();
    let cwd = PathBuf::from("/juego/arte");
    let r = h
        .line("export cofre.png 4", Origin::Cli, Some(cwd.clone()))
        .await
        .unwrap();
    assert!(r.response.is_ok());
    h.batch(
        "export --scale 2 b.png\nexport /abs/c.png",
        Origin::Cli,
        Some(cwd),
    )
    .await
    .unwrap();
    session.close().await;
    let exports = store.exports();
    let got: Vec<_> = exports.iter().map(|e| (e.path.clone(), e.scale)).collect();
    assert_eq!(
        got,
        vec![
            (PathBuf::from("/juego/arte/cofre.png"), 4),
            (PathBuf::from("/juego/arte/b.png"), 2),
            (PathBuf::from("/abs/c.png"), 1),
        ]
    );
    assert_eq!(exports[0].canvas.get(0, 0).unwrap(), Rgba::rgb(255, 0, 0));
}

#[tokio::test]
async fn export_errors_come_back_as_protocol_errors() {
    let (session, store) = spawn(2, 2);
    store.missing_dir("/no/existe");
    let r = session
        .handle
        .line("export /no/existe/a.png", Origin::Cli, None)
        .await
        .unwrap();
    assert_eq!(
        r.response.to_string(),
        "err io open /no/existe/a.png: no such file or directory"
    );
    session.close().await;
}

#[tokio::test]
async fn stop_is_reported_to_the_caller() {
    let (session, _) = spawn(2, 2);
    let r = session
        .handle
        .line("stop", Origin::Cli, None)
        .await
        .unwrap();
    assert!(r.stop);
    assert_eq!(r.response.to_string(), "ok");
    session.close().await;
}

#[tokio::test]
async fn closing_ends_the_task_cleanly() {
    let (session, _) = spawn(2, 2);
    let handle = session.handle.clone();
    let mut events = handle.subscribe();
    handle
        .line("set_pixel 1 1 red", Origin::Cli, None)
        .await
        .unwrap();
    let doc = session.close().await;
    assert_eq!(doc.canvas().get(1, 1).unwrap(), Rgba::rgb(255, 0, 0));
    assert!(!handle.is_open());
    assert_eq!(
        handle.line("undo", Origin::Cli, None).await.unwrap_err(),
        AppError::Closed
    );
    assert_eq!(handle.snapshot().await.unwrap_err(), AppError::Closed);
    // Llegan los eventos pendientes y, al final, Closed (el canal sigue
    // abierto porque `handle` aún existe: se espera al evento, no al cierre).
    wait_closed(&mut events).await;
}

/// Espera (como mucho 5 s) a que llegue `SessionEvent::Closed`.
async fn wait_closed(events: &mut tokio::sync::broadcast::Receiver<SessionEvent>) {
    let wait = async {
        loop {
            match events.recv().await {
                Ok(SessionEvent::Closed) => break,
                Ok(_) => continue,
                Err(e) => panic!("{e:?}"),
            }
        }
    };
    tokio::time::timeout(std::time::Duration::from_secs(5), wait)
        .await
        .expect("debería llegar SessionEvent::Closed");
}

#[tokio::test]
async fn dropping_every_handle_also_ends_the_task() {
    let store = Arc::new(MemoryStore::new());
    let session = Session::spawn(Document::new("x", 2, 2).unwrap(), store);
    let mut events = session.handle.subscribe();
    let Session { handle, .. } = session;
    drop(handle);
    wait_closed(&mut events).await;
}

#[tokio::test]
async fn apply_labeled_is_one_labeled_history_step() {
    let (session, _) = spawn(4, 4);
    let h = &session.handle;
    let stroke = Command::Batch(vec![
        Command::SetPixel {
            x: 0,
            y: 0,
            color: Rgba::rgb(255, 0, 0),
        },
        Command::FillRect {
            x: 1,
            y: 1,
            w: 2,
            h: 1,
            color: Rgba::rgb(255, 0, 0),
        },
    ]);
    let patch = h
        .apply_labeled(stroke, Origin::User, "Lápiz · 3 píxeles")
        .await
        .unwrap();
    assert_eq!(patch.cell_count(), 3);
    let vm = h.view().borrow().clone();
    assert_eq!(vm.history.len(), 1);
    assert_eq!(vm.history[0].label, "Lápiz · 3 píxeles");
    assert_eq!(vm.history[0].origin, Origin::User);
    assert_eq!(vm.last_change.as_ref().unwrap().label, "Lápiz · 3 píxeles");
    // Un comando que falla no deja paso.
    let bad = Command::SetPixel {
        x: 9,
        y: 9,
        color: Rgba::rgb(0, 0, 0),
    };
    assert!(
        h.apply_labeled(bad, Origin::User, "Lápiz · 1 píxel")
            .await
            .is_err()
    );
    assert_eq!(h.view().borrow().history.len(), 1);
    h.line("undo", Origin::User, None).await.unwrap();
    let doc = session.close().await;
    assert!(
        doc.canvas()
            .pixels()
            .iter()
            .all(|p| *p == Rgba::TRANSPARENT)
    );
}

#[tokio::test]
async fn replace_document_swaps_the_document() {
    let (session, _) = spawn(4, 4);
    let h = &session.handle;
    h.line("set_pixel 0 0 red", Origin::Cli, None)
        .await
        .unwrap();
    let mut other = Document::new("otro", 16, 8).unwrap();
    other
        .apply(
            Command::SetPixel {
                x: 15,
                y: 7,
                color: Rgba::rgb(0, 0, 255),
            },
            Origin::User,
        )
        .unwrap();
    h.replace_document(other).await.unwrap();
    let vm = h.view().borrow().clone();
    assert_eq!(vm.name, "otro");
    assert_eq!((vm.canvas.width(), vm.canvas.height()), (16, 8));
    assert_eq!(vm.canvas.get(15, 7).unwrap(), Rgba::rgb(0, 0, 255));
    assert_eq!(vm.revision, 2);
    session.close().await;
}

#[tokio::test]
async fn set_name_renames_without_a_new_revision() {
    let (session, _) = spawn(2, 2);
    let h = &session.handle;
    h.line("set_pixel 0 0 red", Origin::Cli, None)
        .await
        .unwrap();
    h.set_name("cofre").await.unwrap();
    let vm = h.view().borrow().clone();
    assert_eq!(vm.name, "cofre");
    assert_eq!(vm.revision, 1);
    assert_eq!(session.close().await.name, "cofre");
}

/// Notas y versiones son cambios del documento (hay algo que guardar) pero
/// no del dibujo: suben la revisión sin evento de cambio.
#[tokio::test]
async fn notes_and_versions_are_saved_in_the_document() {
    let (session, _) = spawn(2, 2);
    let h = &session.handle;
    let mut events = h.subscribe();
    h.line("set_pixel 0 0 red", Origin::User, None)
        .await
        .unwrap();
    // Lo que publicó el `set_pixel` ya está en el canal.
    while events.try_recv().is_ok() {}
    h.set_notes("falta el tapón").await.unwrap();
    let v1 = h
        .create_version(Origin::Agent { run: 2 }, "Vuelta 1 de 3")
        .await
        .unwrap();
    let v2 = h
        .create_version(Origin::Agent { run: 2 }, "Vuelta 2 de 3")
        .await
        .unwrap();
    assert_eq!((v1, v2), (1, 2));
    let vm = h.view().borrow().clone();
    assert_eq!(vm.revision, 4);
    assert_eq!(vm.versions.len(), 2);
    assert_eq!(vm.versions[1].message, "Vuelta 2 de 3");
    assert!(
        events.try_recv().is_err(),
        "sin eventos de cambio del dibujo"
    );
    let doc = session.close().await;
    assert_eq!(doc.notes, "falta el tapón");
    assert_eq!(doc.versions().len(), 2);
}
