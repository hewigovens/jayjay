use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use super::{RunningJjProcesses, SyncToken};
use crate::types::*;

#[test]
fn close_terminates_a_process_group_that_ignores_sigterm() {
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
    wait_for_running(&processes, 1);

    let started = Instant::now();
    processes.close();
    let output = worker
        .join()
        .expect("command thread should finish")
        .expect("wait for killed command");

    assert!(!output.status.success());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(processes.running_count(), 0);
}

#[test]
fn closed_registry_rejects_new_processes() {
    let processes = RunningJjProcesses::default();
    processes.close();

    let error = run_true(&processes).expect_err("quitting rejects new processes");
    assert!(format!("{error}").contains("quitting"));
}

#[test]
fn cancel_targets_only_the_processes_of_its_action() {
    let processes = RunningJjProcesses::default();
    let pull = processes.sync_token();
    let push = processes.sync_token();
    let pull_worker = spawn_sleeper(&processes, Some(&pull));
    let push_worker = spawn_sleeper(&processes, Some(&push));
    let unbound_worker = spawn_sleeper(&processes, None);
    wait_for_running(&processes, 3);

    pull.cancel();

    let error = pull_worker
        .join()
        .expect("pull thread")
        .expect_err("the pull process should be canceled");
    assert!(matches!(error, CoreError::Canceled), "{error}");
    assert_eq!(processes.running_count(), 2);
    assert!(push.check().is_ok());

    processes.close();
    push_worker
        .join()
        .expect("push thread")
        .expect("the push process is killed, not canceled");
    unbound_worker
        .join()
        .expect("unbound thread")
        .expect("a process outside any action is killed, not canceled");
}

#[test]
fn a_canceled_action_spawns_nothing_more() {
    let processes = RunningJjProcesses::default();
    let sync = processes.sync_token();
    sync.cancel();

    {
        let _enter = sync.enter();
        assert!(matches!(sync.check(), Err(CoreError::Canceled)));
        let refused = run_true(&processes).expect_err("the action's later processes are refused");
        assert!(matches!(refused, CoreError::Canceled), "{refused}");
    }
    run_true(&processes).expect("processes outside the action are unaffected");

    let next = processes.sync_token();
    let _enter = next.enter();
    run_true(&processes).expect("the next action starts clean");
}

#[test]
fn a_live_process_is_canceled_even_after_it_printed_to_stderr() {
    let processes = RunningJjProcesses::default();
    let sync = processes.sync_token();
    let temp_dir = tempfile::tempdir().expect("tempdir");
    let printed = temp_dir.path().join("printed");
    let worker_processes = processes.clone();
    let worker_sync = sync.clone();
    let worker = thread::spawn(move || {
        let _enter = worker_sync.enter();
        let mut command = Command::new("/bin/sh");
        command.args([
            "-c",
            "echo 'remote: banner' >&2; : > \"$0\"; sleep 30",
            printed.to_str().expect("utf-8 path"),
        ]);
        worker_processes.output(&mut command, "chatty command")
    });
    let deadline = Instant::now() + Duration::from_secs(2);
    while !temp_dir.path().join("printed").exists() {
        assert!(Instant::now() < deadline, "process did not print");
        thread::sleep(Duration::from_millis(5));
    }

    sync.cancel();

    let error = worker
        .join()
        .expect("command thread")
        .expect_err("a live process reached by the cancel is canceled");
    assert!(matches!(error, CoreError::Canceled), "{error}");
}

fn run_true(processes: &RunningJjProcesses) -> CoreResult<std::process::Output> {
    processes.output(&mut Command::new("/usr/bin/true"), "true")
}

fn spawn_sleeper(
    processes: &RunningJjProcesses,
    sync: Option<&SyncToken>,
) -> thread::JoinHandle<CoreResult<std::process::Output>> {
    let processes = processes.clone();
    let sync = sync.cloned();
    thread::spawn(move || {
        let _enter = sync.as_ref().map(SyncToken::enter);
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "sleep 30"]);
        processes.output(&mut command, "sleeper")
    })
}

fn wait_for_running(processes: &RunningJjProcesses, count: usize) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while processes.running_count() < count {
        assert!(Instant::now() < deadline, "processes did not start");
        thread::sleep(Duration::from_millis(10));
    }
}
