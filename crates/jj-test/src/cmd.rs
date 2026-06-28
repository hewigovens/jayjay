use std::path::Path;
use std::process::{Command, Output};

pub fn run_command(program: &str, display_args: &[String], command: &mut Command) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("failed to run {program} {display_args:?}: {err}"));

    if !output.status.success() {
        panic!(
            "{program} {display_args:?} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }

    output
}

/// Parse a command's stdout as JSON, panicking with the raw output so failures show what the command actually printed.
pub fn json_stdout(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not valid JSON ({err})\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        )
    })
}

pub fn run_jj(args: &[&str]) -> Output {
    let mut command = Command::new("jj");
    command.args(args);
    let display_args = args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>();
    run_command("jj", &display_args, &mut command)
}

/// Run `jj` rooted at `repo` and panic on non-zero exit.
pub fn run_jj_in(repo: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("jj");
    command.arg("-R").arg(repo).args(args);
    let display_args = std::iter::once("-R".to_string())
        .chain(std::iter::once(repo.display().to_string()))
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect::<Vec<_>>();
    run_command("jj", &display_args, &mut command)
}

pub fn run_git(repo_path: &Path, args: &[&str]) -> Output {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path).args(args);
    let display_args = std::iter::once("-C".to_string())
        .chain(std::iter::once(repo_path.display().to_string()))
        .chain(args.iter().map(|arg| arg.to_string()))
        .collect::<Vec<_>>();
    run_command("git", &display_args, &mut command)
}

/// Build a fresh colocated jj repo at `path` (must not exist yet).
pub fn init_colocated(path: &Path) {
    let mut command = Command::new("jj");
    command.arg("git").arg("init").arg("--colocate").arg(path);
    let display_args = vec![
        "git".to_owned(),
        "init".to_owned(),
        "--colocate".to_owned(),
        path.display().to_string(),
    ];
    run_command("jj", &display_args, &mut command);
}

/// Set a deterministic test identity so commit hashes are reproducible.
pub fn configure_test_user(repo: &Path) {
    run_jj_in(repo, &["config", "set", "--repo", "user.name", "Test User"]);
    run_jj_in(
        repo,
        &["config", "set", "--repo", "user.email", "test@example.com"],
    );
}
