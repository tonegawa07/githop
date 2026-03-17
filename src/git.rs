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
        .map(|output| parse_branch_names(&output))
        .unwrap_or_default();

    let output = run_git(&["branch"])?;
    Ok(parse_branches(&output, &merged_names))
}

fn parse_branch_names(output: &str) -> Vec<String> {
    output
        .lines()
        .map(|l| l.trim().trim_start_matches("* ").to_string())
        .collect()
}

fn parse_branches(output: &str, merged_names: &[String]) -> Vec<Branch> {
    output
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
        .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // --- Unit tests for parse_branches / parse_branch_names ---

    #[test]
    fn parse_branches_detects_current_branch() {
        let output = "  feature-a\n* main\n  feature-b\n";
        let merged = vec!["main".to_string(), "feature-a".to_string()];
        let branches = parse_branches(output, &merged);

        assert_eq!(branches.len(), 3);
        assert!(!branches[0].is_current);
        assert!(branches[1].is_current);
        assert!(!branches[2].is_current);
        assert_eq!(branches[1].name, "main");
    }

    #[test]
    fn parse_branches_detects_merged_status() {
        let output = "  feature-a\n* main\n  feature-b\n";
        let merged = vec!["main".to_string(), "feature-a".to_string()];
        let branches = parse_branches(output, &merged);

        assert!(branches[0].is_merged); // feature-a
        assert!(branches[1].is_merged); // main
        assert!(!branches[2].is_merged); // feature-b
    }

    #[test]
    fn parse_branches_skips_empty_lines() {
        let output = "  feature-a\n\n* main\n  \n";
        let branches = parse_branches(output, &[]);

        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "feature-a");
        assert_eq!(branches[1].name, "main");
    }

    #[test]
    fn parse_branches_empty_output() {
        let branches = parse_branches("", &[]);
        assert!(branches.is_empty());
    }

    #[test]
    fn parse_branches_no_merged() {
        let output = "* main\n  dev\n";
        let branches = parse_branches(output, &[]);

        assert_eq!(branches.len(), 2);
        assert!(!branches[0].is_merged);
        assert!(!branches[1].is_merged);
    }

    #[test]
    fn parse_branch_names_extracts_names() {
        let output = "  feature-a\n* main\n  feature-b\n";
        let names = parse_branch_names(output);

        assert_eq!(names, vec!["feature-a", "main", "feature-b"]);
    }

    // --- Integration tests with temporary git repo ---

    fn setup_temp_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let run = |args: &[&str]| {
            Command::new("git")
                .args(args)
                .current_dir(dir.path())
                .env("GIT_AUTHOR_NAME", "test")
                .env("GIT_AUTHOR_EMAIL", "test@test.com")
                .env("GIT_COMMITTER_NAME", "test")
                .env("GIT_COMMITTER_EMAIL", "test@test.com")
                .output()
                .unwrap()
        };
        run(&["init", "-b", "main"]);
        std::fs::write(dir.path().join("README.md"), "# test").unwrap();
        run(&["add", "."]);
        run(&["commit", "-m", "initial"]);
        dir
    }

    fn run_git_in(dir: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(dir)
            .env("GIT_AUTHOR_NAME", "test")
            .env("GIT_AUTHOR_EMAIL", "test@test.com")
            .env("GIT_COMMITTER_NAME", "test")
            .env("GIT_COMMITTER_EMAIL", "test@test.com")
            .output()
            .with_context(|| format!("failed to run: git {}", args.join(" ")))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("{}", stderr.trim());
        }
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    #[test]
    fn integration_list_branches_in_temp_repo() {
        let dir = setup_temp_repo();
        run_git_in(dir.path(), &["branch", "feature-x"]).unwrap();

        let output = run_git_in(dir.path(), &["branch"]).unwrap();
        let merged_output = run_git_in(dir.path(), &["branch", "--merged"]).unwrap();
        let merged_names = parse_branch_names(&merged_output);
        let branches = parse_branches(&output, &merged_names);

        assert_eq!(branches.len(), 2);
        let main = branches.iter().find(|b| b.name == "main").unwrap();
        assert!(main.is_current);
        assert!(main.is_merged);
        let feature = branches.iter().find(|b| b.name == "feature-x").unwrap();
        assert!(!feature.is_current);
        assert!(feature.is_merged); // no diverged commits, so merged
    }

    #[test]
    fn integration_unmerged_branch_detected() {
        let dir = setup_temp_repo();
        run_git_in(dir.path(), &["branch", "feature-y"]).unwrap();
        run_git_in(dir.path(), &["switch", "feature-y"]).unwrap();
        std::fs::write(dir.path().join("new.txt"), "content").unwrap();
        run_git_in(dir.path(), &["add", "."]).unwrap();
        run_git_in(dir.path(), &["commit", "-m", "diverge"]).unwrap();
        run_git_in(dir.path(), &["switch", "main"]).unwrap();

        let output = run_git_in(dir.path(), &["branch"]).unwrap();
        let merged_output = run_git_in(dir.path(), &["branch", "--merged"]).unwrap();
        let merged_names = parse_branch_names(&merged_output);
        let branches = parse_branches(&output, &merged_names);

        let feature = branches.iter().find(|b| b.name == "feature-y").unwrap();
        assert!(!feature.is_merged);
    }

    #[test]
    fn integration_run_git_fails_outside_repo() {
        let dir = tempfile::tempdir().unwrap();
        let result = Command::new("git")
            .args(["branch"])
            .current_dir(dir.path())
            .output()
            .unwrap();
        assert!(!result.status.success());
    }
}
