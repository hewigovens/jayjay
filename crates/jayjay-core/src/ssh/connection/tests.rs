use super::*;
use jj_test::{init_jj_repo, run_jj_in};
use serde_json::{Value, json};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

struct SshFixture {
    server: Child,
    directory: TempDir,
}
impl SshFixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path();
        for key in ["host", "client"] {
            assert!(
                Command::new("ssh-keygen")
                    .args(["-q", "-t", "ed25519", "-N", "", "-f"])
                    .arg(path.join(key))
                    .status()
                    .unwrap()
                    .success()
            );
        }
        std::fs::copy(path.join("client.pub"), path.join("authorized_keys")).unwrap();
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let user =
            String::from_utf8(Command::new("id").arg("-un").output().unwrap().stdout).unwrap();
        let host_key = std::fs::read_to_string(path.join("host.pub")).unwrap();
        std::fs::write(
            path.join("known_hosts"),
            format!("[127.0.0.1]:{port} {host_key}"),
        )
        .unwrap();
        let remote_home = path.join("remote-home");
        std::fs::create_dir(&remote_home).unwrap();
        let shell = path.join("remote-shell");
        std::fs::write(
            &shell,
            format!(
                "#!/bin/sh\nexport HOME='{}'\nexport PATH={}\nexec /bin/sh -c \"$SSH_ORIGINAL_COMMAND\"\n",
                remote_home.display(),
                shell_words::quote(&std::env::var("PATH").unwrap())
            ),
        )
        .unwrap();
        std::fs::set_permissions(&shell, std::fs::Permissions::from_mode(0o700)).unwrap();
        std::fs::write(path.join("server.conf"), format!("Port {port}\nListenAddress 127.0.0.1\nHostKey {0}/host\nPidFile {0}/pid\nAuthorizedKeysFile {0}/authorized_keys\nPasswordAuthentication no\nKbdInteractiveAuthentication no\nUsePAM no\nStrictModes no\nAllowUsers {1}\nAllowTcpForwarding no\nLogLevel ERROR\nForceCommand {0}/remote-shell\n", path.display(), user.trim())).unwrap();
        std::fs::write(path.join("client.conf"), format!("Host fixture\n HostName 127.0.0.1\n Port {port}\n User {1}\n IdentityFile {0}/client\n UserKnownHostsFile {0}/known_hosts\n IdentitiesOnly yes\n StrictHostKeyChecking yes\n BatchMode yes\n", path.display(), user.trim())).unwrap();
        let mut server = Command::new("/usr/sbin/sshd")
            .args(["-D", "-e", "-f"])
            .arg(path.join("server.conf"))
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            assert!(server.try_wait().unwrap().is_none(), "isolated sshd exited");
            if std::net::TcpStream::connect(("127.0.0.1", port)).is_ok() {
                break;
            }
            assert!(Instant::now() < deadline, "sshd did not listen");
            thread::sleep(Duration::from_millis(20));
        }
        Self { server, directory }
    }
    fn connect(&self) -> Arc<Connection> {
        let helper = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("jayjay-ssh");
        assert!(
            helper.is_file(),
            "build jayjay-ssh before running the live SSH test"
        );
        let mut command = Command::new("ssh");
        command
            .args(["-T", "-a", "-F"])
            .arg(self.directory.path().join("client.conf"))
            .args(["--", "fixture"])
            .arg(format!(
                "exec '{}'",
                helper.display().to_string().replace('\'', "'\\''")
            ));
        Connection::start(command).unwrap()
    }
}
impl Drop for SshFixture {
    fn drop(&mut self) {
        let _ = self.server.kill();
        let _ = self.server.wait();
    }
}

#[test]
#[ignore = "requires a locally runnable sshd and a built jayjay-ssh binary"]
fn real_ssh_disconnect_reports_uncertainty_and_reconnect_reads_persisted_changes() {
    let fixture = SshFixture::new();
    let temp = init_jj_repo();
    let path = temp.path().join("repo");
    let remote = fixture.connect();
    let path_text = path.to_str().unwrap();
    remote
        .request(path_text, "__open", vec![], None)
        .unwrap()
        .result
        .unwrap();
    remote
        .request(
            path_text,
            "describe",
            vec![json!("@"), json!("through real SSH")],
            None,
        )
        .unwrap()
        .result
        .unwrap();
    let marker = temp.path().join("started");
    let git = temp.path().join("git");
    std::fs::write(
        &git,
        format!("#!/bin/sh\ntouch '{}'\nsleep 30\n", marker.display()),
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
    let config = fixture
        .directory
        .path()
        .join("remote-home/.config/jj/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(
        &config,
        format!(
            "[git]\nexecutable-path = {}\n",
            serde_json::to_string(git.to_str().unwrap()).unwrap()
        ),
    )
    .unwrap();
    thread::scope(|scope| {
        let pending = scope.spawn(|| {
            remote.request(
                path_text,
                "git_fetch",
                vec![json!("origin"), Value::Null],
                None,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            if pending.is_finished() {
                panic!(
                    "fetch finished before launching git: {:?}",
                    pending.join().unwrap().and_then(|response| response.result)
                );
            }
            assert!(Instant::now() < deadline, "fetch did not start");
            thread::sleep(Duration::from_millis(20));
        }
        remote.child.lock().unwrap().kill().unwrap();
        let error = pending
            .join()
            .unwrap()
            .err()
            .expect("connection loss must be reported");
        assert!(error.to_string().contains("may have completed"), "{error}");
    });
    assert!(!remote.connected());
    std::fs::remove_file(config).unwrap();
    let reconnected = fixture.connect();
    let log = reconnected
        .request(path_text, "log", vec![json!("@")], None)
        .unwrap()
        .result
        .unwrap();
    assert_eq!(log[0]["description"], "through real SSH");
    reconnected.close();
}

#[test]
#[ignore = "requires a locally runnable sshd and a built jayjay-ssh binary"]
fn real_ssh_facade_deploys_helper_and_reconnects_shared_workspaces() {
    const CHILD: &str = "JAYJAY_SSH_FIXTURE";
    if let Some(directory) = std::env::var_os(CHILD) {
        let directory = std::path::PathBuf::from(directory);
        crate::ssh::set_helper_directory(directory.join("helpers"));
        let location =
            crate::ssh::RemoteLocation::new("fixture", directory.join("repo").to_str().unwrap())
                .unwrap();
        let repo = crate::Repo::open(Path::new(&location.encode())).unwrap();
        repo.describe("@", "deployed helper").unwrap();
        assert_eq!(repo.log("@").unwrap()[0].description, "deployed helper");
        assert!(repo.poll_remote_changes().unwrap());
        assert!(!repo.poll_remote_changes().unwrap());
        std::fs::write(directory.join("repo/polled.txt"), "remote edit").unwrap();
        assert!(repo.poll_remote_changes().unwrap());
        assert!(!repo.poll_remote_changes().unwrap());
        let context = repo.repository_context().unwrap();
        assert!(
            context
                .workspaces
                .unwrap()
                .iter()
                .all(|w| w.path.starts_with("ssh://fixture/"))
        );
        let sibling = directory.join("workspace with spaces");
        let sibling_location = location.qualify(sibling.to_str().unwrap());
        let message = repo
            .workspace_add(&sibling_location, "remote-workspace", "@")
            .unwrap();
        assert!(message.starts_with("Created workspace"));
        let added = repo
            .workspace_list()
            .unwrap()
            .into_iter()
            .find(|workspace| workspace.name == "remote-workspace")
            .unwrap()
            .path;
        let second = crate::Repo::open(Path::new(&added)).unwrap();
        assert_eq!(second.primary_root(), repo.path());
        repo.disconnect().unwrap();
        assert_eq!(
            second.connection_state(),
            crate::ssh::ConnectionState::Disconnected
        );
        assert!(second.describe("@", "must not run offline").is_err());
        second.reconnect().unwrap();
        assert_eq!(
            repo.connection_state(),
            crate::ssh::ConnectionState::Connected
        );
        assert_eq!(repo.log("@").unwrap()[0].description, "deployed helper");
        repo.disconnect().unwrap();
        assert!(
            !repo
                .connection_error()
                .unwrap()
                .contains("may have completed")
        );
        return;
    }
    let fixture = SshFixture::new();
    let directory = fixture.directory.path();
    let repo = directory.join("repo");
    let template = init_jj_repo();
    assert!(
        Command::new("cp")
            .arg("-R")
            .arg(template.path().join("repo"))
            .arg(&repo)
            .status()
            .unwrap()
            .success()
    );
    let binary = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("jayjay-ssh");
    let target = format!(
        "{}-{}",
        std::env::consts::ARCH,
        if cfg!(target_os = "macos") {
            "apple-darwin"
        } else {
            "unknown-linux-musl"
        }
    );
    let helpers = directory.join("helpers").join(target);
    std::fs::create_dir_all(&helpers).unwrap();
    std::fs::copy(binary, helpers.join("jayjay-ssh")).unwrap();
    let bin = directory.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let ssh = bin.join("ssh");
    std::fs::write(
        &ssh,
        format!(
            "#!/bin/sh\nexec /usr/bin/ssh -F '{}' \"$@\"\n",
            directory.join("client.conf").display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut paths = vec![bin];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let status = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "ssh::connection::tests::real_ssh_facade_deploys_helper_and_reconnects_shared_workspaces", "--ignored", "--nocapture"])
        .env(CHILD, directory)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("JAYJAY_SSH_MACHINES_PATH", directory.join("machines.json"))
        .status().unwrap();
    assert!(status.success());
    let deployed = directory.join("remote-home/.local/share/jayjay/helpers");
    assert_eq!(std::fs::read_dir(deployed).unwrap().count(), 1);
}

#[test]
fn slow_machine_setup_does_not_block_other_machines() {
    const CHILD: &str = "JAYJAY_SSH_SLOW_FIXTURE";
    if std::env::var_os(CHILD).is_some() {
        use crate::ssh::machine::MachineConnection;
        thread::spawn(|| MachineConnection::connect("slow"));
        thread::sleep(Duration::from_millis(300));
        let started = Instant::now();
        assert!(MachineConnection::connect("fast").is_err());
        assert!(started.elapsed() < Duration::from_secs(3));
        return;
    }
    let directory = tempfile::tempdir().unwrap();
    let ssh = directory.path().join("ssh");
    std::fs::write(
        &ssh,
        "#!/bin/sh\ncase \" $* \" in *\" -- slow \"*) sleep 10 ;; esac\nexit 1\n",
    )
    .unwrap();
    std::fs::set_permissions(&ssh, std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut paths = vec![directory.path().to_owned()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap()));
    let status = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "ssh::connection::tests::slow_machine_setup_does_not_block_other_machines",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("PATH", std::env::join_paths(paths).unwrap())
        .status()
        .unwrap();
    assert!(status.success());
}
