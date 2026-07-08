use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn exec(mut command: Command) -> io::Error {
    command.exec()
}
