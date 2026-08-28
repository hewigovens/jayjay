use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use super::RunningJjProcesses;

#[test]
fn cancel_terminates_a_process_group_that_ignores_sigterm() {
    let processes = RunningJjProcesses::default();
    let worker_processes = processes.clone();
    let worker = thread::spawn(move || {
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "trap '' TERM; /bin/sh -c 'trap \"\" TERM; while :; do sleep 1; done' & wait",
        ]);
        worker_processes.output(&mut command, "stubborn command")
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    while processes.running_count() == 0 {
        assert!(Instant::now() < deadline, "command did not start");
        thread::sleep(Duration::from_millis(10));
    }

    let started = Instant::now();
    processes.cancel();
    let output = worker
        .join()
        .expect("command thread should finish")
        .expect("wait for killed command");

    assert!(!output.status.success());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(processes.running_count(), 0);
}

#[test]
fn canceled_registry_rejects_new_processes() {
    let processes = RunningJjProcesses::default();
    processes.cancel();

    let error = processes
        .output(&mut Command::new("/usr/bin/true"), "new command")
        .expect_err("quitting should reject new processes");
    assert!(format!("{error}").contains("quitting"));
}
