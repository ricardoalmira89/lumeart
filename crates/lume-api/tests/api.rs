//! Tests de la API local. Casi todos llaman al `Router` directamente (sin
//! sockets); `server_on_loopback` arranca el servidor en 127.0.0.1.

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use lume_api::{ApiServer, FAILED_HEADER, read_state_file, router, write_state_file};
use lume_app::ports::CancellationToken;
use lume_app::testing::MemoryStore;
use lume_app::{DocumentStore, Session};
use lume_domain::{Canvas, Document, Rgba, SessionInfo};
use tower::ServiceExt;

const TOKEN: &str = "secreto-de-prueba";

struct Fixture {
    app: Router,
    session: Session,
    store: Arc<MemoryStore>,
    stop: CancellationToken,
}

fn fixture(w: i64, h: i64) -> Fixture {
    let store = Arc::new(MemoryStore::new());
    let session = Session::spawn(Document::new("api", w, h).unwrap(), store.clone());
    let stop = CancellationToken::new();
    let app = router(session.handle.clone(), store.clone(), TOKEN, stop.clone());
    Fixture {
        app,
        session,
        store,
        stop,
    }
}

fn request(method: &str, uri: &str, body: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(Body::from(body.to_string())).unwrap()
}

async fn send(
    app: &Router,
    method: &str,
    uri: &str,
    body: &str,
) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let response = app
        .clone()
        .oneshot(request(method, uri, body, Some(TOKEN)))
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, headers, bytes)
}

async fn send_text(app: &Router, method: &str, uri: &str, body: &str) -> (StatusCode, String) {
    let (status, _, bytes) = send(app, method, uri, body).await;
    (status, String::from_utf8(bytes).unwrap())
}

#[tokio::test]
async fn requests_without_the_right_token_get_401() {
    let fx = fixture(4, 4);
    for token in [None, Some("otro"), Some("")] {
        let response = fx
            .app
            .clone()
            .oneshot(request("GET", "/v1/canvas.png", "", token))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{token:?}");
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(&body[..], b"err unauthorized missing or invalid token\n");
    }
    // Otro esquema distinto de Bearer tampoco vale.
    let req = Request::builder()
        .uri("/v1/events")
        .header("authorization", format!("Basic {TOKEN}"))
        .body(Body::empty())
        .unwrap();
    assert_eq!(
        fx.app.clone().oneshot(req).await.unwrap().status(),
        StatusCode::UNAUTHORIZED
    );
    // Con el token, sí.
    let (status, _) = send_text(&fx.app, "POST", "/v1/commands", "get_pixel 0 0").await;
    assert_eq!(status, StatusCode::OK);
    fx.session.close().await;
}

#[tokio::test]
async fn commands_answer_one_protocol_line_each() {
    let fx = fixture(4, 4);
    let (status, body) = send_text(
        &fx.app,
        "POST",
        "/v1/commands",
        "set_pixel 1 1 red\nget_pixel 1 1\n\nset_pixel 9 9 red\nnope\n",
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        "ok\nok #ff0000ff\nerr out_of_bounds pixel (9,9) outside canvas\nerr invalid_command unknown command \"nope\"\n"
    );
    let (_, body) = send_text(&fx.app, "POST", "/v1/commands", "  ").await;
    assert_eq!(body, "err invalid_command command is required\n");
    fx.session.close().await;
}

/// El mismo guion que `lume-domain/tests/fixtures/batch-1.txt`, con la salida
/// capturada de la CLI real de pxcli.
#[tokio::test]
async fn batch_returns_the_same_lines_as_pxcli() {
    let fx = fixture(4, 4);
    let script = include_str!("../../lume-domain/tests/fixtures/batch-1.txt");
    let expected = include_str!("../../lume-domain/tests/fixtures/batch-1.pxcli.txt");
    let stdout = expected.split("--- stderr ---").next().unwrap();
    let (status, headers, body) = send(
        &fx.app,
        "POST",
        "/v1/commands?batch=1&cwd=/BATCHDIR",
        script,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // La salida de pxcli se capturó en Linux; en Windows las rutas llevan `\`.
    let body = String::from_utf8(body).unwrap();
    let body = if cfg!(windows) {
        body.replace('\\', "/")
    } else {
        body
    };
    assert_eq!(body, stdout);
    assert_eq!(headers[FAILED_HEADER], "8");
    // El export relativo se resolvió en la carpeta del cliente.
    assert_eq!(
        fx.store.exports()[0].path,
        std::path::PathBuf::from("/BATCHDIR/out.png")
    );

    let (status, body) = send_text(&fx.app, "POST", "/v1/commands?batch=1", "# nada\n").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body, "err invalid_args batch is empty\n");
    fx.session.close().await;
}

#[tokio::test]
async fn canvas_png_returns_the_current_canvas() {
    let fx = fixture(3, 2);
    send_text(&fx.app, "POST", "/v1/commands", "set_pixel 2 1 #12345678").await;
    let mut expected = Canvas::new(3, 2).unwrap();
    expected
        .set(2, 1, Rgba::new(0x12, 0x34, 0x56, 0x78))
        .unwrap();

    let (status, headers, body) = send(&fx.app, "GET", "/v1/canvas.png", "").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(headers["content-type"], "image/png");
    assert_eq!(body, fx.store.encode_png(&expected, 1).unwrap());
    let (_, _, body) = send(&fx.app, "GET", "/v1/canvas.png?scale=3", "").await;
    assert_eq!(body, fx.store.encode_png(&expected, 3).unwrap());

    let (status, body) = send_text(&fx.app, "GET", "/v1/canvas.png?scale=0", "").await;
    assert_eq!(
        (status, body.as_str()),
        (
            StatusCode::BAD_REQUEST,
            "err invalid_args scale must be between 1 and 64\n"
        )
    );
    let (status, body) = send_text(&fx.app, "GET", "/v1/canvas.png?scale=x", "").await;
    assert_eq!(
        (status, body.as_str()),
        (
            StatusCode::BAD_REQUEST,
            "err invalid_args scale must be an integer\n"
        )
    );
    fx.session.close().await;
}

/// Lee del flujo SSE hasta tener `n` eventos (como mucho 5 s).
async fn read_events(body: &mut Body, n: usize) -> Vec<(String, serde_json::Value)> {
    let mut buffer = String::new();
    let mut events = Vec::new();
    let read = async {
        while events.len() < n {
            let frame = body.frame().await.expect("el flujo sigue abierto").unwrap();
            if let Ok(data) = frame.into_data() {
                buffer.push_str(std::str::from_utf8(&data).unwrap());
            }
            while let Some(end) = buffer.find("\n\n") {
                let block: String = buffer.drain(..end + 2).collect();
                let mut name = String::new();
                let mut data = String::new();
                for line in block.lines() {
                    if let Some(v) = line.strip_prefix("event: ") {
                        name = v.to_string();
                    } else if let Some(v) = line.strip_prefix("data: ") {
                        data = v.to_string();
                    }
                }
                if !name.is_empty() {
                    events.push((name, serde_json::from_str(&data).unwrap()));
                }
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), read)
        .await
        .expect("llegan los eventos");
    events
}

#[tokio::test]
async fn sse_events_arrive_after_each_command() {
    let fx = fixture(4, 4);
    let response = fx
        .app
        .clone()
        .oneshot(request("GET", "/v1/events", "", Some(TOKEN)))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.headers()["content-type"], "text/event-stream");
    let mut body = response.into_body();

    send_text(&fx.app, "POST", "/v1/commands", "fill_rect 0 0 2 2 blue").await;
    let events = read_events(&mut body, 2).await;
    assert_eq!(events[0].0, "change");
    assert_eq!(events[0].1["revision"], 1);
    assert_eq!(events[0].1["cells"], 4);
    assert_eq!(events[0].1["origin"], "cli");
    assert_eq!(events[0].1["label"], "fill_rect 0 0 2 2 blue");
    assert_eq!(events[1].0, "command");
    assert_eq!(events[1].1["response"], "ok");

    send_text(&fx.app, "POST", "/v1/commands", "get_pixel 0 0").await;
    let events = read_events(&mut body, 1).await;
    assert_eq!(events[0].0, "command");
    assert_eq!(events[0].1["response"], "ok #0000ffff");

    send_text(&fx.app, "POST", "/v1/undo", "").await;
    let events = read_events(&mut body, 2).await;
    assert_eq!(events[0].0, "change");
    assert_eq!(events[0].1["label"], "undo");

    fx.session.close().await;
    let events = read_events(&mut body, 1).await;
    assert_eq!(events[0].0, "closed");
}

#[tokio::test]
async fn new_undo_redo_and_stop_routes() {
    let fx = fixture(4, 4);
    assert_eq!(
        send_text(&fx.app, "POST", "/v1/session/new?size=8x4", "")
            .await
            .1,
        "ok 8x4\n"
    );
    assert_eq!(
        send_text(&fx.app, "POST", "/v1/undo", "").await.1,
        "err no_history nothing to undo\n"
    );
    send_text(&fx.app, "POST", "/v1/commands", "set_pixel 7 3 red").await;
    assert_eq!(send_text(&fx.app, "POST", "/v1/undo", "").await.1, "ok\n");
    assert_eq!(send_text(&fx.app, "POST", "/v1/redo", "").await.1, "ok\n");
    assert_eq!(
        send_text(&fx.app, "POST", "/v1/commands", "get_pixel 7 3")
            .await
            .1,
        "ok #ff0000ff\n"
    );
    let (status, body) = send_text(&fx.app, "POST", "/v1/session/new?size=grande", "").await;
    assert_eq!(
        (status, body.as_str()),
        (
            StatusCode::BAD_REQUEST,
            "err invalid_args invalid size \"grande\": expected WxH\n"
        )
    );
    assert_eq!(
        send_text(&fx.app, "POST", "/v1/session/new?size=0x4", "")
            .await
            .1,
        "err invalid_args width and height must be between 1 and 512\n"
    );

    assert!(!fx.stop.is_cancelled());
    assert_eq!(send_text(&fx.app, "POST", "/v1/stop", "").await.1, "ok\n");
    assert!(fx.stop.is_cancelled());
    fx.session.close().await;

    // La línea `stop` del protocolo también pide parar.
    let fx = fixture(2, 2);
    assert_eq!(
        send_text(&fx.app, "POST", "/v1/commands", "stop").await.1,
        "ok\n"
    );
    assert!(fx.stop.is_cancelled());
    fx.session.close().await;
}

#[tokio::test]
async fn closed_session_gives_503() {
    let fx = fixture(2, 2);
    let app = fx.app.clone();
    fx.session.close().await;
    let (status, body) = send_text(&app, "POST", "/v1/commands", "get_pixel 0 0").await;
    assert_eq!(
        (status, body.as_str()),
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "err closed session closed\n"
        )
    );
}

#[test]
fn state_file_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sub/session.json");
    let info = SessionInfo {
        port: 41234,
        pid: 77,
        token: "abc".into(),
        version: "0.1.0".into(),
    };
    write_state_file(&path, &info).unwrap();
    assert_eq!(read_state_file(&path).unwrap(), info);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
    assert_eq!(info.base_url(), "http://127.0.0.1:41234");
}

#[test]
fn tokens_are_random_and_long() {
    let a = lume_api::generate_token();
    let b = lume_api::generate_token();
    assert_eq!(a.len(), 64);
    assert_ne!(a, b);
    assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
}

/// Arranca el servidor de verdad (solo en 127.0.0.1) y le habla por TCP.
#[tokio::test]
async fn server_on_loopback() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let store = Arc::new(MemoryStore::new());
    let session = Session::spawn(Document::new("api", 2, 2).unwrap(), store.clone());
    let server = ApiServer::start(session.handle.clone(), store)
        .await
        .unwrap();
    let info = server.info();
    assert_eq!(server.addr.ip().to_string(), "127.0.0.1");
    assert_eq!(info.pid, std::process::id());

    let mut stream = tokio::net::TcpStream::connect(server.addr).await.unwrap();
    let body = "set_pixel 1 1 red";
    let req = format!(
        "POST /v1/commands HTTP/1.1\r\nHost: localhost\r\nAuthorization: Bearer {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        info.token,
        body.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.ends_with("\r\n\r\nok\n"), "{response}");

    let stop = server.stop_requested();
    server.shutdown().await.unwrap();
    assert!(stop.is_cancelled());
    session.close().await;
}

/// Con `run=N` (la CLI lo añade si hay `LUMEART_RUN`), los cambios quedan
/// como del agente en esa ejecución; sin él, como de la CLI.
#[tokio::test]
async fn commands_from_an_agent_run_are_marked_as_agent() {
    use lume_domain::Origin;
    let fx = fixture(4, 4);
    send_text(&fx.app, "POST", "/v1/commands", "set_pixel 0 0 red").await;
    send_text(&fx.app, "POST", "/v1/commands?run=7", "set_pixel 1 0 red").await;
    send_text(
        &fx.app,
        "POST",
        "/v1/commands?batch=1&cwd=/tmp&run=7",
        "set_pixel 2 0 red\nset_pixel 3 0 red",
    )
    .await;
    let history = fx.session.handle.view().borrow().history.clone();
    let origins: Vec<Origin> = history.iter().map(|h| h.origin).collect();
    assert_eq!(
        origins,
        [
            Origin::Cli,
            Origin::Agent { run: 7 },
            Origin::Agent { run: 7 }
        ]
    );
    fx.session.close().await;
}
