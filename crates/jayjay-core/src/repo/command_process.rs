#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(all(test, unix))]
mod tests;

use std::collections::HashSet;
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

#[cfg(unix)]
use unix::{configure_process_group, terminate_process_group};
#[cfg(windows)]
use windows::{configure_process_group, terminate_process_group};

use crate::types::*;

/// A stray `jj git fetch` has survived SIGTERM in the field, so quit escalates to SIGKILL after this grace.
const TERMINATE_GRACE: Duration = Duration::from_millis(200);

#[derive(Clone, Default)]
pub(super) struct RunningJjProcesses {
    state: Arc<Mutex<ProcessState>>,
}

#[derive(Default)]
struct ProcessState {
    closed: bool,
    pids: HashSet<u32>,
}

impl RunningJjProcesses {
    pub(super) fn output(&self, command: &mut Command, context: &str) -> CoreResult<Output> {
        let internal = |error: std::io::Error| CoreError::Internal {
            message: format!("{context}: {error}"),
        };
        configure_process_group(command);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Spawn under the lock so no process can slip in between the closed check and the kill sweep.
        let child = {
            let mut state = self.state();
            if state.closed {
                return Err(CoreError::Internal {
                    message: format!("{context}: canceled because JayJay is quitting"),
                });
            }
            let child = command.spawn().map_err(internal)?;
            state.pids.insert(child.id());
            child
        };
        let pid = child.id();
        let output = child.wait_with_output();
        self.state().pids.remove(&pid);
        output.map_err(internal)
    }

    pub(super) fn cancel(&self) {
        let pids = {
            let mut state = self.state();
            state.closed = true;
            state.pids.clone()
        };
        if pids.is_empty() {
            return;
        }
        for pid in &pids {
            terminate_process_group(*pid, false);
        }
        std::thread::sleep(TERMINATE_GRACE);
        let state = self.state();
        for pid in pids.intersection(&state.pids) {
            terminate_process_group(*pid, true);
        }
    }

    fn state(&self) -> MutexGuard<'_, ProcessState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }

    #[cfg(test)]
    fn running_count(&self) -> usize {
        self.state().pids.len()
    }
}
