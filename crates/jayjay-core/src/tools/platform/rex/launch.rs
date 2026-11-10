use std::thread;

use super::cli::Cli;

/// Rex confirms a launch only after several CLI calls, so `fallback` also covers a failure found once this has returned.
pub(crate) fn run(cwd: &str, command: &str, fallback: fn(&str, &str) -> bool) -> bool {
    let Some(cli) = Cli::find() else {
        return fallback(cwd, command);
    };
    let (cwd, command) = (cwd.to_owned(), command.to_owned());
    thread::spawn(move || {
        if !cli.run(&cwd, &command) {
            log::warn!("rex could not run the command in {cwd}; using the fallback terminal");
            fallback(&cwd, &command);
        }
    });
    true
}
