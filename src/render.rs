use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_replay(level: &Path, playback: &Path) -> Result<()> {
    let workspace_root = workspace_root()?;
    let status = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("gsnake-cli")
        .arg("--")
        .arg("--level-file")
        .arg(level)
        .arg("--input-file")
        .arg(playback)
        .current_dir(workspace_root)
        .status()
        .with_context(|| "Failed to run gsnake-cli replay")?;

    if status.success() {
        Ok(())
    } else {
        bail!("Replay failed with exit code {status}")
    }
}

pub fn run_render(level: &Path, playback: &Path) -> Result<()> {
    ensure_command("asciinema")?;
    ensure_svg_term()?;

    let cast_path = playback.with_extension("cast");
    let svg_path = infer_svg_path(playback)?;
    if let Some(parent) = svg_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let workspace_root = workspace_root()?;
    let status = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("gsnake-cli")
        .arg("--")
        .arg("--level-file")
        .arg(level)
        .arg("--input-file")
        .arg(playback)
        .arg("--record")
        .arg("--record-output")
        .arg(&cast_path)
        .current_dir(workspace_root)
        .status()
        .with_context(|| "Failed to run gsnake-cli with recording")?;

    if !status.success() {
        bail!("Recording failed with exit code {status}");
    }

    let svg_term = svg_term_command()?;
    let status = Command::new(svg_term)
        .arg("--in")
        .arg(&cast_path)
        .arg("--out")
        .arg(&svg_path)
        .status()
        .with_context(|| "Failed to run svg-term")?;

    if !status.success() {
        bail!("SVG render failed with exit code {status}");
    }

    Ok(())
}

fn ensure_command(command: &str) -> Result<()> {
    let status = Command::new(command).arg("--version").status();
    if matches!(status, Ok(status) if status.success()) {
        Ok(())
    } else {
        bail!("Required command '{command}' is not available in PATH")
    }
}

fn ensure_svg_term() -> Result<()> {
    if svg_term_command()?.is_empty() {
        bail!("svg-term is not available in PATH. Install svg-term-cli")
    }
    Ok(())
}

fn svg_term_command() -> Result<String> {
    for candidate in ["svg-term", "svg-term-cli"] {
        if matches!(Command::new(candidate).arg("--version").status(), Ok(status) if status.success()) {
            return Ok(candidate.to_string());
        }
    }
    Ok(String::new())
}

fn infer_svg_path(playback: &Path) -> Result<PathBuf> {
    let mut output = PathBuf::new();
    let mut replaced = false;
    for component in playback.components() {
        let component_str = component.as_os_str();
        if component_str == "playbacks" && !replaced {
            output.push("renders");
            replaced = true;
        } else {
            output.push(component_str);
        }
    }

    if !replaced {
        return Ok(playback.with_extension("svg"));
    }

    Ok(output.with_extension("svg"))
}

fn workspace_root() -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve workspace root"))?;
    Ok(root.to_path_buf())
}
