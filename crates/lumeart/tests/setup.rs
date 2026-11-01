//! `lumeart setup` con el binario real, una carpeta personal de prueba y
//! `claude`/`codex` falsos que se comportan como los de verdad: `claude mcp
//! add` escribe en `~/.claude.json` y falla si ya existe; `codex mcp add`
//! escribe en `~/.codex/config.toml`. Nunca toca la configuración real.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Output;

const SKILL: &str = include_str!("../../../skills/lumeart/SKILL.md");

const FAKE_CLAUDE: &str = r#"#!/bin/sh
echo "claude $*" >> "$HOME/llamadas.log"
if [ "$1 $2" = "mcp add" ]; then
  if [ -f "$HOME/.claude.json" ]; then
    IFS= read -r content < "$HOME/.claude.json"
    case "$content" in *'"lumeart"'*)
      echo "MCP server lumeart already exists in user config" >&2
      exit 1;;
    esac
  fi
  printf '{"mcpServers":{"lumeart":{"type":"stdio","command":"%s","args":["%s"],"env":{}}}}' "$7" "$8" > "$HOME/.claude.json"
elif [ "$1 $2" = "mcp remove" ]; then
  printf '{"mcpServers":{}}' > "$HOME/.claude.json"
fi
"#;

const FAKE_CODEX: &str = r#"#!/bin/sh
echo "codex $*" >> "$HOME/llamadas.log"
if [ "$1 $2" = "mcp add" ]; then
  /bin/mkdir -p "$HOME/.codex"
  printf '[mcp_servers.lumeart]\ncommand = "%s"\nargs = ["%s"]\n' "$5" "$6" > "$HOME/.codex/config.toml"
elif [ "$1 $2" = "mcp remove" ]; then
  printf '' > "$HOME/.codex/config.toml"
fi
"#;

struct Home {
    dir: tempfile::TempDir,
    bin: PathBuf,
}

impl Home {
    fn new(claude: bool, codex: bool) -> Home {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        for (on, name, script) in [
            (claude, "claude", FAKE_CLAUDE),
            (codex, "codex", FAKE_CODEX),
        ] {
            if on {
                let path = bin.join(name);
                std::fs::write(&path, script).unwrap();
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        Home { dir, bin }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn setup(&self, args: &[&str]) -> Output {
        std::process::Command::new(env!("CARGO_BIN_EXE_lumeart"))
            .arg("setup")
            .args(args)
            .env("HOME", self.path())
            .env("PATH", &self.bin)
            .env_remove("CLAUDE_CONFIG_DIR")
            .env_remove("CODEX_HOME")
            .env_remove("USERPROFILE")
            .current_dir(self.path())
            .output()
            .unwrap()
    }

    fn calls(&self) -> String {
        std::fs::read_to_string(self.path().join("llamadas.log")).unwrap_or_default()
    }

    fn claude_skill(&self) -> PathBuf {
        self.path().join(".claude/skills/lumeart/SKILL.md")
    }

    fn agents_skill(&self) -> PathBuf {
        self.path().join(".agents/skills/lumeart/SKILL.md")
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn exe() -> String {
    env!("CARGO_BIN_EXE_lumeart").to_string()
}

#[test]
fn dry_run_says_exactly_what_it_would_do_and_changes_nothing() {
    let home = Home::new(true, true);
    let out = home.setup(&["--dry-run"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let claude = home.bin.join("claude");
    let codex = home.bin.join("codex");
    assert_eq!(
        text(&out.stdout),
        format!(
            "Claude Code ({claude}):\n  Ejecutaría: {claude} mcp add --scope user lumeart -- {exe} mcp\n  Crearía {cskill}\nCodex ({codex}):\n  Ejecutaría: {codex} mcp add lumeart -- {exe} mcp\n  Crearía {askill}\n--dry-run: no se ha cambiado nada.\n",
            claude = claude.display(),
            codex = codex.display(),
            exe = exe(),
            cskill = home.claude_skill().display(),
            askill = home.agents_skill().display(),
        )
    );
    assert_eq!(home.calls(), "", "no ejecutó nada");
    assert!(!home.claude_skill().exists());
    assert!(!home.agents_skill().exists());
    assert!(!home.path().join(".claude.json").exists());
}

#[test]
fn running_twice_does_not_duplicate_anything() {
    let home = Home::new(true, true);
    let first = home.setup(&[]);
    assert!(first.status.success(), "{}", text(&first.stdout));
    let stdout = text(&first.stdout);
    assert!(stdout.contains("  Hecho: "), "{stdout}");
    assert!(stdout.contains("  Skill creada: "), "{stdout}");
    assert_eq!(
        home.calls(),
        format!(
            "claude mcp add --scope user lumeart -- {exe} mcp\ncodex mcp add lumeart -- {exe} mcp\n",
            exe = exe()
        )
    );
    assert_eq!(std::fs::read_to_string(home.claude_skill()).unwrap(), SKILL);
    assert_eq!(std::fs::read_to_string(home.agents_skill()).unwrap(), SKILL);

    // La segunda vez no ejecuta nada ni reescribe nada.
    let calls = home.calls();
    let second = home.setup(&[]);
    assert!(second.status.success(), "{}", text(&second.stdout));
    assert_eq!(home.calls(), calls);
    let stdout = text(&second.stdout);
    assert_eq!(
        stdout.matches("MCP: ya está registrado").count(),
        2,
        "{stdout}"
    );
    assert_eq!(
        stdout.matches("Skill: ya está al día").count(),
        2,
        "{stdout}"
    );
    assert!(stdout.ends_with("Todo estaba ya listo.\n"), "{stdout}");
    let toml = std::fs::read_to_string(home.path().join(".codex/config.toml")).unwrap();
    assert_eq!(toml.matches("[mcp_servers.lumeart]").count(), 1);

    // Y --dry-run tampoco tiene nada que hacer.
    let dry = home.setup(&["--dry-run"]);
    assert!(!text(&dry.stdout).contains("Ejecutaría"));
    assert!(!text(&dry.stdout).contains("Crearía"));
}

#[test]
fn an_old_registration_or_skill_is_updated() {
    let home = Home::new(true, true);
    std::fs::write(
        home.path().join(".claude.json"),
        r#"{"mcpServers":{"lumeart":{"command":"/viejo/lumeart","args":["mcp"]}}}"#,
    )
    .unwrap();
    std::fs::create_dir_all(home.claude_skill().parent().unwrap()).unwrap();
    std::fs::write(home.claude_skill(), "skill antigua").unwrap();
    let out = home.setup(&[]);
    assert!(out.status.success(), "{}", text(&out.stdout));
    let stdout = text(&out.stdout);
    assert!(
        stdout.contains("MCP: estaba registrado con otro comando (/viejo/lumeart mcp)"),
        "{stdout}"
    );
    assert!(stdout.contains("Skill actualizada: "), "{stdout}");
    assert!(
        home.calls()
            .starts_with("claude mcp remove --scope user lumeart\nclaude mcp add")
    );
    assert_eq!(std::fs::read_to_string(home.claude_skill()).unwrap(), SKILL);
    let config = std::fs::read_to_string(home.path().join(".claude.json")).unwrap();
    assert!(config.contains(&exe()), "{config}");
}

#[test]
fn a_missing_agent_is_reported_and_the_other_one_is_set_up() {
    let home = Home::new(false, true);
    let out = home.setup(&[]);
    assert!(out.status.success(), "{}", text(&out.stdout));
    let stdout = text(&out.stdout);
    assert!(
        stdout.starts_with(
            "Claude Code: no está instalado (no se encontró `claude`); me lo salto.\nCodex ("
        ),
        "{stdout}"
    );
    assert!(!home.claude_skill().exists());
    assert!(home.agents_skill().exists());
    assert!(home.calls().starts_with("codex mcp add"));

    let home = Home::new(true, false);
    let out = home.setup(&[]);
    assert!(out.status.success());
    assert!(
        text(&out.stdout)
            .contains("Codex: no está instalado (no se encontró `codex`); me lo salto."),
        "{}",
        text(&out.stdout)
    );
    assert!(home.claude_skill().exists());
    assert!(!home.agents_skill().exists());

    // Sin ninguno de los dos, lo dice y sale con error.
    let home = Home::new(false, false);
    let out = home.setup(&[]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stdout).contains("No se encontró ni Claude Code ni Codex"));
}

#[test]
fn a_failing_command_is_reported() {
    let home = Home::new(true, false);
    // Un `claude` que siempre falla.
    std::fs::write(
        home.bin.join("claude"),
        "#!/bin/sh\necho 'sin permiso para escribir' >&2\nexit 3\n",
    )
    .unwrap();
    let out = home.setup(&[]);
    assert_eq!(out.status.code(), Some(1));
    let stdout = text(&out.stdout);
    assert!(stdout.contains("  Falló: "), "{stdout}");
    assert!(stdout.contains("(sin permiso para escribir)"), "{stdout}");
    // La skill se copia igualmente.
    assert!(home.claude_skill().exists());
}
