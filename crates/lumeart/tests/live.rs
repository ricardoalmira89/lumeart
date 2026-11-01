//! Un comando enviado por la API llega a la ventana sin interacción: la
//! sesión publica un `ViewModel` nuevo y la ventana pide repintarse.

use std::sync::Arc;
use std::time::{Duration, Instant};

use lume_api::ApiServer;
use lume_app::Session;
use lume_app::ports::CancellationToken;
use lume_app::testing::MemoryStore;
use lume_domain::{Document, Rgba};

#[test]
fn a_command_sent_through_the_api_repaints_the_window() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let session = runtime.block_on(async {
        Session::spawn(Document::new("directo", 32, 32).unwrap(), store.clone())
    });
    let server = runtime
        .block_on(ApiServer::start(session.handle.clone(), store))
        .unwrap();
    let info = server.info();

    // La ventana (sin abrirla): un contexto de egui que ya pintó un fotograma.
    let ctx = eframe::egui::Context::default();
    // egui pide algún fotograma más al empezar (fuentes); hasta quedar en reposo.
    for _ in 0..10 {
        let mut output = ctx.run_ui(Default::default(), |_| {});
        // Sin ventana no hay quien suba las texturas: se descartan.
        output.textures_delta.clear();
        if !ctx.has_requested_repaint() {
            break;
        }
    }
    assert!(!ctx.has_requested_repaint(), "en reposo no se repinta");
    lume_ui::repaint_on_changes(
        runtime.handle(),
        &session.handle,
        ctx.clone(),
        CancellationToken::new(),
    );

    // Un cliente cualquiera (la CLI, otro agente) dibuja por la API.
    let response = lume_cli::http::request(
        info.port,
        &info.token,
        "POST",
        "/v1/commands",
        b"set_pixel 21 24 #5b2aa6",
    )
    .unwrap();
    assert_eq!(response.text(), "ok\n");

    let deadline = Instant::now() + Duration::from_secs(5);
    while !ctx.has_requested_repaint() {
        assert!(Instant::now() < deadline, "la ventana no pidió repintarse");
        std::thread::sleep(Duration::from_millis(10));
    }
    // Y el ViewModel que pintará ya trae el píxel y el cambio para resaltar.
    let vm = session.handle.view().borrow().clone();
    assert_eq!(vm.canvas.get(21, 24).unwrap(), Rgba::rgb(0x5b, 0x2a, 0xa6));
    let change = vm.last_change.as_ref().unwrap();
    assert_eq!(change.label, "set_pixel 21 24 #5b2aa6");
    assert_eq!(change.origin, lume_domain::Origin::Cli);

    runtime.block_on(async {
        server.shutdown().await.unwrap();
        session.close().await;
    });
}
