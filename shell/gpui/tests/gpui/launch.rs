use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct ProcessGroup(Child);

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        unsafe { libc::kill(-(self.0.id() as i32), libc::SIGKILL) };
        let _ = self.0.wait();
    }
}

#[test]
fn gui_launch_returns_and_survives_terminal_interrupt() {
    let temp = tempfile::tempdir().unwrap();
    let cwd = temp.path().join("repo with spaces");
    fs::create_dir(&cwd).unwrap();
    let appimage = temp.path().join("JayJay.AppImage");
    let record = temp.path().join("launch");
    fs::write(
        &appimage,
        "#!/bin/sh\nprintf '%s\\n' \"$$\" \"$PWD\" \"$@\" >\"$LAUNCH_RECORD\"\nexec sleep 30\n",
    )
    .unwrap();
    fs::set_permissions(&appimage, fs::Permissions::from_mode(0o755)).unwrap();
    let status = temp.path().join("status");
    let mut terminal = ProcessGroup(
        Command::new("sh")
            .args(["-c", "\"$1\" .; echo $? >\"$2\"; exec sleep 30", "launcher"])
            .arg(env!("CARGO_BIN_EXE_jayjay-gpui"))
            .arg(&status)
            .current_dir(&cwd)
            .env("HOME", temp.path())
            .env("XDG_CONFIG_HOME", temp.path())
            .env("APPIMAGE", &appimage)
            .env("LAUNCH_RECORD", &record)
            .env("DISPLAY", "invalid-display")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("ZED_HEADLESS")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(10);
    let launch = loop {
        if let Ok(launch) = fs::read_to_string(&record)
            && launch.lines().count() == 5
            && fs::read_to_string(&status).is_ok_and(|value| value == "0\n")
        {
            break launch;
        }
        assert!(Instant::now() < deadline, "GUI launcher did not return");
        std::thread::sleep(Duration::from_millis(10));
    };
    let lines: Vec<_> = launch.lines().collect();
    let pid: i32 = lines[0].parse().unwrap();
    assert_eq!(
        &lines[1..],
        &[cwd.to_str().unwrap(), "--foreground", "--", "."]
    );
    assert_eq!(unsafe { libc::getsid(pid) }, pid);
    assert_eq!(unsafe { libc::getpgid(pid) }, pid);
    for fd in 0..=2 {
        assert_eq!(
            fs::read_link(format!("/proc/{pid}/fd/{fd}")).unwrap(),
            std::path::Path::new("/dev/null")
        );
    }
    assert_eq!(
        unsafe { libc::kill(-(terminal.0.id() as i32), libc::SIGINT) },
        0
    );
    assert!(!terminal.0.wait().unwrap().success());
    assert_eq!(
        unsafe { libc::kill(pid, 0) },
        0,
        "GUI died with the terminal"
    );
    unsafe { libc::kill(pid, libc::SIGKILL) };
}

#[test]
fn failed_gui_spawn_reports_an_error() {
    let temp = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_jayjay-gpui"))
        .env("APPIMAGE", temp.path().join("missing.AppImage"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).starts_with("error: failed to launch JayJay:"));
}
