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

/// `run_git` output as trimmed text, for ids and paths that tests compare.
pub fn git_stdout(repo_path: &Path, args: &[&str]) -> String {
    String::from_utf8(run_git(repo_path, args).stdout)
        .expect("utf8 git output")
        .trim()
        .to_owned()
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

/// Adds `[fix.tools.fixer]`, which pipes files matching `patterns` (a TOML array) through `script`.
pub fn configure_fix_tool(repo: &Path, script: &str, patterns: &str) {
    let tool = repo.join("fix-tool.sh");
    std::fs::write(&tool, script).expect("write fix tool");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755))
            .expect("make fix tool executable");
    }
    for (key, value) in [
        ("fix.tools.fixer.command", r#"["$root/fix-tool.sh"]"#),
        ("fix.tools.fixer.patterns", patterns),
    ] {
        run_jj_in(repo, &["config", "set", "--repo", key, value]);
    }
}

/// Set a deterministic test identity so commit hashes are reproducible.
pub fn configure_test_user(repo: &Path) {
    run_jj_in(repo, &["config", "set", "--repo", "user.name", "Test User"]);
    run_jj_in(
        repo,
        &["config", "set", "--repo", "user.email", "test@example.com"],
    );
}
