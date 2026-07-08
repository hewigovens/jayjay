use std::io;
use std::process::Command;

pub fn exec(mut command: Command) -> io::Error {
    match command.status() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => error,
    }
}
