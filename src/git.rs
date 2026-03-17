use anyhow::{bail, Context, Result};
use std::process::Command;

pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub is_merged: bool,
}

fn run_git(args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .with_context(|| format!("failed to run: git {}", args.join(" ")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let msg = stderr.trim();
        if msg.is_empty() {
            bail!("git {} exited with {}", args.join(" "), output.status);
        }
        bail!("{}", msg);
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn try_run_git(args: &[&str]) -> Option<String> {
    run_git(args).ok()
}

pub fn list_branches() -> Result<Vec<Branch>> {
    let merged_names: Vec<String> = try_run_git(&["branch", "--merged"])
        .map(|output| {
            output
                .lines()
                .map(|l| l.trim().trim_start_matches("* ").to_string())
                .collect()
        })
        .unwrap_or_default();

    let output = run_git(&["branch"])?;
    let branches = output
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let is_current = line.starts_with('*');
            let name = line.trim().trim_start_matches("* ").to_string();
            let is_merged = merged_names.iter().any(|m| m == &name);
            Branch {
                name,
                is_current,
                is_merged,
            }
        })
        .collect();
    Ok(branches)
}

pub fn switch_branch(name: &str) -> Result<()> {
    run_git(&["switch", name]).with_context(|| {
        format!(
            "Could not switch to '{}'. Do you have uncommitted changes?",
            name
        )
    })?;
    Ok(())
}

pub fn delete_branch(name: &str, force: bool) -> Result<()> {
    let flag = if force { "-D" } else { "-d" };
    run_git(&["branch", flag, name])
        .with_context(|| format!("Could not delete branch '{}'", name))?;
    Ok(())
}

pub fn create_branch(name: &str) -> Result<()> {
    run_git(&["branch", name])
        .with_context(|| format!("Could not create branch '{}'. Does it already exist?", name))?;
    Ok(())
}

pub fn rename_branch(old: &str, new: &str) -> Result<()> {
    run_git(&["branch", "-m", old, new])
        .with_context(|| format!("Could not rename '{}' to '{}'", old, new))?;
    Ok(())
}

pub fn get_log(branch: &str, count: usize) -> Result<Vec<String>> {
    let output = run_git(&[
        "log",
        branch,
        &format!("-{}", count),
        "--oneline",
        "--no-decorate",
    ])?;
    Ok(output.lines().map(|l| l.to_string()).collect())
}

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;

    let (cmd, args) = clipboard_command();
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("Clipboard not available. Is '{}' installed?", cmd))?;
    child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
    child.wait()?;
    Ok(())
}

fn clipboard_command() -> (&'static str, &'static [&'static str]) {
    if cfg!(target_os = "macos") {
        ("pbcopy", &[])
    } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
        ("wl-copy", &[])
    } else {
        ("xclip", &["-selection", "clipboard"])
    }
}
