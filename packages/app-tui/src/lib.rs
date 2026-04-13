use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, anyhow, bail};
use gunmetal_storage::AppPaths;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceSnapshot {
    pub state: String,
    pub running: bool,
    pub url: String,
    pub pid: Option<u32>,
}

pub fn run(paths: &AppPaths, service: ServiceSnapshot) -> Result<()> {
    let script = resolve_tui_entry()?;
    let mut child = Command::new("bun");
    child
        .arg("run")
        .arg(&script)
        .env("GUNMETAL_HOME", &paths.root)
        .env("GUNMETAL_TUI_BASE_URL", &service.url)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let status = child
        .status()
        .with_context(|| "failed to launch Bun for the OpenTUI surface")?;
    if status.success() {
        return Ok(());
    }

    bail!("OpenTUI exited with status {status}");
}

fn resolve_tui_entry() -> Result<PathBuf> {
    if let Ok(value) = std::env::var("GUNMETAL_TUI_ENTRY") {
        let path = PathBuf::from(value);
        if path.exists() {
            return Ok(path);
        }
    }

    let bundled = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("opentui.js");
    if bundled.exists() {
        return Ok(bundled);
    }

    Err(anyhow!(
        "OpenTUI entry not found. expected `packages/app-tui/src/opentui.js`."
    ))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    #[test]
    fn resolves_bundled_tui_entry() {
        let path = super::resolve_tui_entry().unwrap();
        assert!(path.ends_with(PathBuf::from("packages/app-tui/src/opentui.js")));
    }
}
