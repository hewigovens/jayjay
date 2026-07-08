mod platform;

use std::ffi::OsString;
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

    let mut command = Command::new(&app_executable);
    command.args(arguments);
    let error = platform::exec(command);

    eprintln!("error: failed to run {}: {error}", app_executable.display());
    std::process::exit(127);
}
