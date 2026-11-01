//! Cliente HTTP/1.1 mínimo para la API local (`127.0.0.1`): sin TLS ni
//! dependencias. Una conexión por petición (`Connection: close`).

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Respuesta HTTP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Valor de una cabecera (sin distinguir mayúsculas).
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Error al hablar con la API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// No hay nadie escuchando en el puerto.
    Connect(String),
    /// La conexión se cortó o la respuesta no es HTTP válido.
    Io(String),
}

/// Codifica un valor para la parte de consulta de una URL.
pub fn encode_query(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Hace una petición a `127.0.0.1:<port>` (espera la respuesta hasta 120 s).
pub fn request(
    port: u16,
    token: &str,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<HttpResponse, HttpError> {
    request_with_timeout(port, token, method, path, body, Duration::from_secs(120))
}

/// Como [`request`], con un tiempo máximo de espera de la respuesta.
pub fn request_with_timeout(
    port: u16,
    token: &str,
    method: &str,
    path: &str,
    body: &[u8],
    timeout: Duration,
) -> Result<HttpResponse, HttpError> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let mut stream = TcpStream::connect_timeout(&addr, timeout.min(Duration::from_secs(5)))
        .map_err(|e| HttpError::Connect(e.to_string()))?;
    let io = |e: std::io::Error| HttpError::Io(e.to_string());
    stream.set_read_timeout(Some(timeout)).map_err(io)?;
    stream.set_write_timeout(Some(timeout)).map_err(io)?;
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nAuthorization: Bearer {token}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).map_err(io)?;
    stream.write_all(body).map_err(io)?;
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).map_err(io)?;
    parse_response(&raw)
}

/// Interpreta una respuesta HTTP/1.1 completa (con `Content-Length`,
/// `Transfer-Encoding: chunked` o hasta el final de la conexión).
pub fn parse_response(raw: &[u8]) -> Result<HttpResponse, HttpError> {
    let bad = |m: &str| HttpError::Io(format!("invalid HTTP response: {m}"));
    let end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| bad("no header end"))?;
    let head = std::str::from_utf8(&raw[..end]).map_err(|_| bad("headers are not UTF-8"))?;
    let mut lines = head.split("\r\n");
    let status_line = lines.next().ok_or_else(|| bad("empty"))?;
    let status = status_line
        .split(' ')
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .ok_or_else(|| bad("no status"))?;
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    let mut response = HttpResponse {
        status,
        headers,
        body: Vec::new(),
    };
    let rest = &raw[end + 4..];
    response.body = if response
        .header("transfer-encoding")
        .is_some_and(|v| v.eq_ignore_ascii_case("chunked"))
    {
        decode_chunked(rest).ok_or_else(|| bad("bad chunked body"))?
    } else if let Some(len) = response
        .header("content-length")
        .and_then(|v| v.parse::<usize>().ok())
    {
        rest.get(..len)
            .ok_or_else(|| bad("body shorter than Content-Length"))?
            .to_vec()
    } else {
        rest.to_vec()
    };
    Ok(response)
}

fn decode_chunked(mut data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = data.windows(2).position(|w| w == b"\r\n")?;
        let size_text = std::str::from_utf8(&data[..line_end]).ok()?;
        let size = usize::from_str_radix(size_text.split(';').next()?.trim(), 16).ok()?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Some(out);
        }
        out.extend_from_slice(data.get(..size)?);
        data = data.get(size + 2..)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_content_length_and_chunked() {
        let r = parse_response(
            b"HTTP/1.1 200 OK\r\ncontent-length: 3\r\nX-Lumeart-Failed: 2\r\n\r\nok\nbasura",
        )
        .unwrap();
        assert_eq!((r.status, r.text().as_str()), (200, "ok\n"));
        assert_eq!(r.header("x-lumeart-failed"), Some("2"));
        let r = parse_response(b"HTTP/1.1 401 Unauthorized\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nerr \r\n3\r\nyes\r\n0\r\n\r\n").unwrap();
        assert_eq!((r.status, r.text().as_str()), (401, "err yes"));
        let r = parse_response(b"HTTP/1.1 200 OK\r\n\r\nhasta el final").unwrap();
        assert_eq!(r.text(), "hasta el final");
        assert!(parse_response(b"basura").is_err());
    }

    #[test]
    fn query_encoding() {
        assert_eq!(
            encode_query("/home/ana/mi juego/ñ"),
            "/home/ana/mi%20juego/%C3%B1"
        );
        assert_eq!(encode_query("C:\\x&y=z"), "C%3A%5Cx%26y%3Dz");
    }
}
