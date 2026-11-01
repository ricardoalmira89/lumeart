//! Servidor HTTP falso para los tests de los proveedores: escucha en
//! 127.0.0.1 (sin salir del equipo), contesta las respuestas dadas en orden y
//! apunta lo que recibe.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Una petición recibida por el servidor falso.
#[derive(Debug, Clone)]
pub struct Received {
    pub head: String,
    pub body: Value,
}

impl Received {
    pub fn header(&self, name: &str) -> Option<String> {
        self.head.lines().find_map(|line| {
            let (k, v) = line.split_once(':')?;
            k.trim()
                .eq_ignore_ascii_case(name)
                .then(|| v.trim().to_string())
        })
    }
}

/// Servidor HTTP mínimo en 127.0.0.1 que contesta las respuestas dadas, en
/// orden, y apunta lo que recibe.
pub struct FakeApi {
    pub url: String,
    pub received: Arc<Mutex<Vec<Received>>>,
}

pub fn fake_api(rt: &tokio::runtime::Runtime, replies: Vec<(u16, &'static str)>) -> FakeApi {
    let listener = rt
        .block_on(tokio::net::TcpListener::bind("127.0.0.1:0"))
        .unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let received = Arc::new(Mutex::new(Vec::new()));
    let log = received.clone();
    let mut replies: VecDeque<_> = replies.into();
    rt.spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            let mut data = Vec::new();
            let mut buf = [0u8; 8192];
            let (head, body) = loop {
                let n = socket.read(&mut buf).await.unwrap_or(0);
                if n == 0 {
                    break (String::new(), Vec::new());
                }
                data.extend_from_slice(&buf[..n]);
                if let Some(end) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&data[..end]).to_string();
                    let length = head
                        .lines()
                        .find_map(|l| {
                            let (k, v) = l.split_once(':')?;
                            k.eq_ignore_ascii_case("content-length")
                                .then(|| v.trim().parse::<usize>().ok())?
                        })
                        .unwrap_or(0);
                    let mut body = data[end + 4..].to_vec();
                    while body.len() < length {
                        let n = socket.read(&mut buf).await.unwrap_or(0);
                        if n == 0 {
                            break;
                        }
                        body.extend_from_slice(&buf[..n]);
                    }
                    break (head, body);
                }
            };
            log.lock().unwrap().push(Received {
                head,
                body: serde_json::from_slice(&body).unwrap_or(Value::Null),
            });
            let (status, text) = replies
                .pop_front()
                .unwrap_or((500, r#"{"error":{"message":"sin respuesta grabada"}}"#));
            let response = format!(
                "HTTP/1.1 {status} X\r\ncontent-type: application/json\r\ncontent-length: {}\r\nretry-after: 0\r\nconnection: close\r\n\r\n{text}",
                text.len()
            );
            let _ = socket.write_all(response.as_bytes()).await;
            let _ = socket.shutdown().await;
        }
    });
    FakeApi { url, received }
}
