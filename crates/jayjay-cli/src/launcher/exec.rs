use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::process::Command;

use super::app;

pub(crate) fn exec_app<I>(arguments: I) -> !
where
    I: IntoIterator<Item = OsString>,
{
    let Some(app_executable) = app::find_app_executable() else {
        eprintln!("error: JayJay executable not found");
        eprintln!("Install JayJay.app or build with: just build");
        std::process::exit(127);
    };

    let error = Command::new(&app_executable).args(arguments).exec();
    eprintln!("error: failed to run {}: {error}", app_executable.display());
    std::process::exit(127);
}
