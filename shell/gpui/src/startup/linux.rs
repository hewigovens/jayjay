use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use jayjay_core::tools::detach_stdio;

pub(super) fn detach(path: Option<&Path>) -> io::Result<()> {
    // Restart the AppImage so its runtime owns a new mount after the original launcher exits.
    let executable = env::var_os("APPIMAGE")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .map_or_else(env::current_exe, Ok)?;
    let mut command = Command::new(executable);
    command.arg("--foreground");
    if let Some(path) = path {
        command.arg("--").arg(path);
    }
    detach_stdio(&mut command);
    // SAFETY: setsid is async-signal-safe; the child performs no allocation or locking before exec.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command.spawn()?;
    Ok(())
}
