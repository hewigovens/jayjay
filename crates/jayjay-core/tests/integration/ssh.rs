use jayjay_core::{ChangeInfo, DiffHunk, JayError, ssh::RemoteLocation};
use jj_test::{init_jj_repo, run_jj_in};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

struct Helper {
    child: Child,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Helper {
    fn new() -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_jayjay-ssh"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = BufReader::new(child.stdout.take().unwrap());
        let mut helper = Self {
            child,
            input,
            output,
        };
        let hello = helper.read();
        assert_eq!(hello["protocol"], 1);
        helper
    }
    fn read(&mut self) -> Value {
        let mut line = String::new();
        self.output.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
    fn send(&mut self, id: u64, path: &std::path::Path, method: &str, args: Value) {
        serde_json::to_writer(
            &mut self.input,
            &json!({"id":id,"path":path,"method":method,"args":args}),
        )
        .unwrap();
        self.input.write_all(b"\n").unwrap();
        self.input.flush().unwrap();
    }
    fn call(
        &mut self,
        id: u64,
        path: &std::path::Path,
        method: &str,
        args: Value,
    ) -> Result<Value, JayError> {
        self.send(id, path, method, args);
        let response = self.read();
        assert_eq!(response["id"], id);
        serde_json::from_value(response["result"].clone()).unwrap()
    }
}
impl Drop for Helper {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
fn helper_operates_on_the_actual_workspace_and_preserves_typed_failures() {
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    std::fs::write(path.join("name with spaces.txt"), "before\n").unwrap();
    run_jj_in(&path, &["describe", "-m", "original"]);
    let mut helper = Helper::new();
    let opened = helper.call(1, &path, "__open", json!([])).unwrap();
    let root = jayjay_core::Repo::open(&path)
        .unwrap()
        .path()
        .to_string_lossy()
        .into_owned();
    // Windows hosts send `C:\dir` as `/C:/dir`.
    let root = if cfg!(windows) {
        format!("/{}", root.replace('\\', "/"))
    } else {
        root
    };
    assert_eq!(opened["root"], root);
    std::fs::write(path.join("name with spaces.txt"), "after\n").unwrap();
    helper
        .call(2, &path, "refresh_working_copy", json!([]))
        .unwrap();
    helper
        .call(
            3,
            &path,
            "describe",
            json!(["@", "edited over SSH protocol"]),
        )
        .unwrap();
    let changes: Vec<ChangeInfo> =
        serde_json::from_value(helper.call(4, &path, "log", json!(["@"])).unwrap()).unwrap();
    assert_eq!(changes[0].description, "edited over SSH protocol");
    let hunk: DiffHunk = serde_json::from_value(
        helper
            .call(5, &path, "show_file", json!(["@", "name with spaces.txt"]))
            .unwrap(),
    )
    .unwrap();
    assert!(hunk.new.content.unwrap().contains("after"));
    let error = helper
        .call(
            6,
            &path,
            "describe",
            json!(["this_revision_does_not_exist", "unchanged"]),
        )
        .unwrap_err();
    assert!(matches!(error, JayError::RevNotFound { .. }), "{error}");
    assert!(
        helper
            .call(7, &path, "arbitrary_shell", json!(["touch marker"]))
            .is_err()
    );
    assert!(!path.join("marker").exists());
    assert_eq!(
        helper.call(8, &path, "__browse", json!([])).unwrap()["path"],
        root
    );
    let uri = RemoteLocation::new("DevBox", "/home/dev/project with spaces")
        .unwrap()
        .encode();
    assert_eq!(
        jayjay_core::repositories::normalize_repository_path(std::path::Path::new(&uri))
            .to_string_lossy(),
        uri
    );
}

#[cfg(unix)]
#[test]
fn helper_acknowledges_cancel_after_the_remote_fetch_process_exits() {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{Duration, Instant};
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let started = temp.path().join("git-started");
    let git = temp.path().join("git");
    std::fs::write(
        &git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", started.display()),
    )
    .unwrap();
    std::fs::set_permissions(&git, std::fs::Permissions::from_mode(0o755)).unwrap();
    run_jj_in(
        &path,
        &[
            "git",
            "remote",
            "add",
            "origin",
            "https://example.invalid/repo.git",
        ],
    );
    let mut helper = Helper::new();
    helper.call(0, &path, "__open", json!([])).unwrap();
    run_jj_in(
        &path,
        &[
            "config",
            "set",
            "--repo",
            "git.executable-path",
            git.to_str().unwrap(),
        ],
    );
    let began = Instant::now();
    helper.send(1, &path, "git_fetch", json!(["origin", null]));
    let deadline = Instant::now() + Duration::from_secs(10);
    while !started.exists() {
        assert!(Instant::now() < deadline, "fetch did not reach git");
        std::thread::sleep(Duration::from_millis(20));
    }
    helper.send(1, &path, "__cancel", json!([]));
    let response = helper.read();
    assert!(
        began.elapsed() < Duration::from_secs(5),
        "cancellation took too long"
    );
    let result: Result<Value, JayError> =
        serde_json::from_value(response["result"].clone()).unwrap();
    assert!(matches!(result, Err(JayError::Canceled)), "{result:?}");
    assert!(helper.call(2, &path, "log", json!(["@"])).is_ok());
}

#[test]
fn helper_initializes_a_missing_repository_in_place() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("plain folder");
    std::fs::create_dir(&path).unwrap();
    let mut helper = Helper::new();
    let missing = helper.call(1, &path, "__open", json!([])).unwrap_err();
    assert!(
        matches!(missing, JayError::RepoNotFound { .. }),
        "{missing}"
    );
    helper.call(2, &path, "__init", json!([])).unwrap();
    helper.call(3, &path, "__open", json!([])).unwrap();
    assert!(helper.call(4, &path, "__init", json!([])).is_err());
}
