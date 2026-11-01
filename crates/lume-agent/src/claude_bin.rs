//! Encontrar el ejecutable `claude` (Claude Code), como `exec.LookPath` en
//! pxcli, más los sitios donde lo deja su instalador aunque no esté en el PATH.
//! También `codex` (Codex CLI), para `lumeart setup`.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// Ruta de `claude`, o `None` si no está instalado.
pub fn find_claude() -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    find_claude_in(&path, home.as_deref())
}

/// [`find_claude`] con el PATH y la carpeta personal dados (para tests).
pub fn find_claude_in(path: &OsStr, home: Option<&Path>) -> Option<PathBuf> {
    let extra: Vec<PathBuf> = home
        // Instalación local de Claude Code y la carpeta de binarios del usuario.
        .map(|h| {
            vec![
                h.join(".claude").join("local"),
                h.join(".local").join("bin"),
            ]
        })
        .unwrap_or_default();
    find_program_in("claude", path, &extra)
}

/// Ruta de `codex` (Codex CLI), o `None` si no está instalado.
pub fn find_codex() -> Option<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    let extra: Vec<PathBuf> = home
        .map(|h| vec![h.join(".local").join("bin")])
        .unwrap_or_default();
    find_program_in("codex", &path, &extra)
}

/// Busca el ejecutable `name` en el PATH y después en `extra` (en Windows,
/// con `.exe`, `.cmd` o `.bat`).
pub fn find_program_in(name: &str, path: &OsStr, extra: &[PathBuf]) -> Option<PathBuf> {
    let names: Vec<String> = if cfg!(windows) {
        ["exe", "cmd", "bat"]
            .iter()
            .map(|ext| format!("{name}.{ext}"))
            .collect()
    } else {
        vec![name.to_string()]
    };
    let mut dirs: Vec<PathBuf> = std::env::split_paths(path)
        // Una entrada vacía sería la carpeta actual: no se busca ahí.
        .filter(|d| d.is_absolute())
        .collect();
    dirs.extend(extra.iter().cloned());
    dirs.iter()
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| is_executable(candidate))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fake_claude(dir: &Path) -> PathBuf {
        std::fs::create_dir_all(dir).unwrap();
        let name = if cfg!(windows) {
            "claude.exe"
        } else {
            "claude"
        };
        let file = dir.join(name);
        std::fs::write(&file, "#!/bin/sh\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        file
    }

    #[test]
    fn finds_claude_in_the_path_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let (a, b, empty) = (
            tmp.path().join("a"),
            tmp.path().join("b"),
            tmp.path().join("c"),
        );
        std::fs::create_dir_all(&empty).unwrap();
        fake_claude(&a);
        let second = fake_claude(&b);
        let path = std::env::join_paths([&empty, &b, &a]).unwrap();
        assert_eq!(find_claude_in(&path, None), Some(second));
    }

    #[test]
    fn missing_claude_is_none() {
        let tmp = tempfile::tempdir().unwrap();
        let path = std::env::join_paths([tmp.path()]).unwrap();
        assert_eq!(find_claude_in(&path, Some(tmp.path())), None);
        assert_eq!(find_claude_in(OsStr::new(""), None), None);
    }

    #[test]
    fn falls_back_to_the_installer_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let local = fake_claude(&tmp.path().join(".claude").join("local"));
        assert_eq!(
            find_claude_in(OsStr::new(""), Some(tmp.path())),
            Some(local)
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_file_that_cannot_run_is_not_claude() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempfile::tempdir().unwrap();
        let file = fake_claude(tmp.path());
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let path = std::env::join_paths([tmp.path()]).unwrap();
        assert_eq!(find_claude_in(&path, None), None);
    }
}
