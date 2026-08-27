use std::os::unix::process::CommandExt as _;
use std::process::Command;

pub(super) fn configure_process_group(command: &mut Command) {
    command.process_group(0);
}

pub(super) fn terminate_process_group(pid: u32, force: bool) {
    let Ok(pid) = i32::try_from(pid) else {
        return;
    };
    let signal = if force { libc::SIGKILL } else { libc::SIGTERM };
    // SAFETY: a negative PID targets the dedicated process group created for this child.
    unsafe {
        libc::kill(-pid, signal);
    }
}
