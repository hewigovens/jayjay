mod args;
mod launcher;

use clap::Parser;
use std::ffi::OsString;

use crate::args::Cli;

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    if should_run_in_app(args.get(1)) {
        launcher::exec_app(args.into_iter().skip(1));
    }

    let cli = Cli::parse_from(args);
    if cli.show_version {
        println!("jayjay {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    launcher::launch_app(cli.repo, cli.path);
}

fn should_run_in_app(first_arg: Option<&OsString>) -> bool {
    let Some(first_arg) = first_arg.and_then(|arg| arg.to_str()) else {
        return false;
    };
    first_arg == "review"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_app_owned_cli_commands() {
        assert!(should_run_in_app(Some(&OsString::from("review"))));
    }

    #[test]
    fn launcher_arguments_stay_in_rust() {
        for arg in ["--version", "-v", "--repo", ".", "--help", "README.md"] {
            assert!(!should_run_in_app(Some(&OsString::from(arg))));
        }
    }
}
