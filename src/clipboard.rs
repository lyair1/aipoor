use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    for program in ["pbcopy", "wl-copy", "xclip", "xsel"] {
        if try_copy(program, text)? {
            return Ok(());
        }
    }

    bail!("no supported clipboard command found (tried pbcopy, wl-copy, xclip, xsel)")
}

fn try_copy(program: &str, text: &str) -> Result<bool> {
    let mut command = match program {
        "pbcopy" => {
            let mut cmd = Command::new(program);
            cmd.stdin(Stdio::piped());
            cmd
        }
        "wl-copy" => {
            let mut cmd = Command::new(program);
            cmd.stdin(Stdio::piped());
            cmd
        }
        "xclip" => {
            let mut cmd = Command::new(program);
            cmd.args(["-selection", "clipboard"]).stdin(Stdio::piped());
            cmd
        }
        "xsel" => {
            let mut cmd = Command::new(program);
            cmd.args(["--clipboard", "--input"]).stdin(Stdio::piped());
            cmd
        }
        _ => return Ok(false),
    };

    let spawn = command.spawn();
    let mut child = match spawn {
        Ok(child) => child,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err).with_context(|| format!("failed to launch {program}")),
    };

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .with_context(|| format!("failed to write clipboard payload to {program}"))?;
    }

    let status = child
        .wait()
        .with_context(|| format!("failed waiting for {program}"))?;

    Ok(status.success())
}
