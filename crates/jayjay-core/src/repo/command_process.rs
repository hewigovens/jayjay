#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(all(test, unix))]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Read;
use std::process::{Child, ChildStderr, ChildStdout, Command, ExitStatus, Output, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;

#[cfg(unix)]
use unix::{configure_process_group, terminate_process_group};
#[cfg(windows)]
use windows::{configure_process_group, terminate_process_group};

use crate::types::*;

/// A stray `jj git fetch` has survived SIGTERM in the field, so termination escalates to SIGKILL after this grace.
const TERMINATE_GRACE: Duration = Duration::from_millis(200);
const REAP_POLL: Duration = Duration::from_millis(10);

thread_local! {
    static CURRENT_SYNC: RefCell<Option<SyncToken>> = const { RefCell::new(None) };
}

#[derive(Clone, Default)]
pub(super) struct RunningJjProcesses {
    state: Arc<Mutex<ProcessState>>,
}

#[derive(Default)]
struct ProcessState {
    closed: bool,
    running: HashMap<u32, RunningProcess>,
}

struct RunningProcess {
    child: Child,
    sync: Option<SyncToken>,
    signaled: bool,
}

/// Cancellation handle for one fetch or push: the shell creates it before the action starts, and every jj process the action spawns is bound to it.
#[derive(Clone)]
pub struct SyncToken {
    canceled: Arc<AtomicBool>,
    processes: RunningJjProcesses,
}

impl SyncToken {
    /// Returns as soon as the cancel is latched and SIGTERM is sent; the SIGKILL escalation runs in the background.
    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::SeqCst);
        let targets: Vec<u32> = self
            .processes
            .state()
            .running
            .iter_mut()
            .filter_map(|(pid, process)| {
                let mine = process.sync.as_ref().is_some_and(|sync| sync.is(self));
                // Checked and signaled under the registry lock, so a process that exits on its own cannot be reaped in between and reported as canceled.
                let live = matches!(process.child.try_wait(), Ok(None));
                (mine && live).then(|| {
                    process.signaled = true;
                    terminate_process_group(*pid, false);
                    *pid
                })
            })
            .collect();
        self.processes.escalate(targets, false);
    }

    pub(crate) fn check(&self) -> CoreResult<()> {
        if self.is_canceled() {
            return Err(CoreError::Canceled);
        }
        Ok(())
    }

    pub(crate) fn enter(&self) -> SyncEnter {
        let previous = CURRENT_SYNC.with(|current| current.replace(Some(self.clone())));
        SyncEnter { previous }
    }

    fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::SeqCst)
    }

    fn is(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.canceled, &other.canceled)
    }
}

pub(crate) struct SyncEnter {
    previous: Option<SyncToken>,
}

impl Drop for SyncEnter {
    fn drop(&mut self) {
        let previous = self.previous.take();
        CURRENT_SYNC.with(|current| *current.borrow_mut() = previous);
    }
}

impl RunningJjProcesses {
    pub(super) fn sync_token(&self) -> SyncToken {
        SyncToken {
            canceled: Arc::default(),
            processes: self.clone(),
        }
    }

    pub(super) fn output(&self, command: &mut Command, context: &str) -> CoreResult<Output> {
        let internal = |error: std::io::Error| CoreError::Internal {
            message: format!("{context}: {error}"),
        };
        let sync = CURRENT_SYNC.with(|current| current.borrow().clone());
        configure_process_group(command);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Checked and spawned under the lock so no process can slip in between a cancel or close and its kill sweep.
        let (pid, stdout, stderr) = {
            let mut state = self.state();
            if state.closed {
                return Err(CoreError::Internal {
                    message: format!("{context}: canceled because JayJay is quitting"),
                });
            }
            if sync.as_ref().is_some_and(SyncToken::is_canceled) {
                return Err(CoreError::Canceled);
            }
            let mut child = command.spawn().map_err(internal)?;
            let pid = child.id();
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();
            let process = RunningProcess {
                child,
                sync,
                signaled: false,
            };
            state.running.insert(pid, process);
            (pid, stdout, stderr)
        };
        let pipes = read_pipes(stdout, stderr);
        let (status, signaled) = self.reap(pid).map_err(internal)?;
        let (stdout, stderr) = pipes.map_err(internal)?;
        if signaled && !status.success() {
            return Err(CoreError::Canceled);
        }
        Ok(Output {
            status,
            stdout,
            stderr,
        })
    }

    fn reap(&self, pid: u32) -> std::io::Result<(ExitStatus, bool)> {
        loop {
            {
                let mut state = self.state();
                let Some(process) = state.running.get_mut(&pid) else {
                    return Err(std::io::Error::other("process left the registry early"));
                };
                match process.child.try_wait() {
                    Ok(Some(status)) => {
                        let signaled = process.signaled;
                        state.running.remove(&pid);
                        return Ok((status, signaled));
                    }
                    Ok(None) => {}
                    Err(error) => {
                        state.running.remove(&pid);
                        return Err(error);
                    }
                }
            }
            thread::sleep(REAP_POLL);
        }
    }

    pub(super) fn close(&self) {
        let targets: Vec<u32> = {
            let mut state = self.state();
            state.closed = true;
            for pid in state.running.keys() {
                terminate_process_group(*pid, false);
            }
            state.running.keys().copied().collect()
        };
        self.escalate(targets, true);
    }

    fn escalate(&self, pids: Vec<u32>, wait_for_grace: bool) {
        if pids.is_empty() {
            return;
        }
        let processes = self.clone();
        let escalate = move || {
            thread::sleep(TERMINATE_GRACE);
            let state = processes.state();
            for pid in pids.iter().filter(|pid| state.running.contains_key(pid)) {
                terminate_process_group(*pid, true);
            }
        };
        if wait_for_grace {
            escalate();
        } else {
            thread::spawn(escalate);
        }
    }

    fn state(&self) -> MutexGuard<'_, ProcessState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    #[cfg(test)]
    fn running_count(&self) -> usize {
        self.state().running.len()
    }
}

fn read_pipes(
    stdout: Option<ChildStdout>,
    stderr: Option<ChildStderr>,
) -> std::io::Result<(Vec<u8>, Vec<u8>)> {
    let stderr_reader = thread::spawn(move || read_to_end(stderr));
    let stdout = read_to_end(stdout)?;
    let stderr = stderr_reader
        .join()
        .map_err(|_| std::io::Error::other("stderr reader panicked"))??;
    Ok((stdout, stderr))
}

fn read_to_end(pipe: Option<impl Read>) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if let Some(mut pipe) = pipe {
        pipe.read_to_end(&mut bytes)?;
    }
    Ok(bytes)
}
