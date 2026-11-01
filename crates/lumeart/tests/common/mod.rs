//! Ayudantes para los tests que usan el binario real `lumeart`. Cada test usa
//! su propio archivo de estado en una carpeta temporal (`--session`), así que
//! no toca la sesión del usuario. La API escucha solo en 127.0.0.1.

#![allow(dead_code)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};

pub struct Lumeart {
    pub state: PathBuf,
    pub cwd: PathBuf,
}

impl Lumeart {
    pub fn new(dir: &Path) -> Self {
        Self {
            state: dir.join("estado").join("session.json"),
            cwd: dir.to_path_buf(),
        }
    }

    fn command(&self, args: &[&str]) -> std::process::Command {
        let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"));
        command
            .arg("--session")
            .arg(&self.state)
            .args(args)
            .current_dir(&self.cwd)
            .env_remove("LUMEART_SESSION");
        command
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    /// Como `run`, sin pantalla (Linux: sin `DISPLAY` ni `WAYLAND_DISPLAY`),
    /// para probar lo que pasa cuando no se puede abrir la ventana.
    pub fn run_without_display(&self, args: &[&str]) -> Output {
        self.command(args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()
            .unwrap()
    }

    /// Como `run`, con este texto en la entrada estándar.
    pub fn run_with_stdin(&self, args: &[&str], stdin: &str) -> Output {
        let mut child = self
            .command(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(stdin.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

/// Para la sesión al terminar el test, aunque falle.
impl Drop for Lumeart {
    fn drop(&mut self) {
        if self.state.exists() {
            let _ = self.run(&["stop"]);
        }
    }
}

pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
