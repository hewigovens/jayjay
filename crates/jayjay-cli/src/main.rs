mod args;
mod launcher;

use clap::Parser;
use std::ffi::OsString;
use std::path::Path;

use crate::args::{Cli, Command};
use jayjay_primitives::{JAYJAY_CONFIG_COMMAND, JAYJAY_REVIEW_COMMAND, JAYJAY_TOOL_COMMAND};

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    if let Some(arguments) = app_arguments(&args) {
        launcher::exec_app(arguments);
    }

    let cli = Cli::parse_from(args);
    if cli.show_version {
        println!("jayjay {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if matches!(cli.command, Some(Command::Config)) {
        print!("{}", jayjay_primitives::JJ_TOOL_CONFIG);
        return;
    }

    launcher::launch_app(cli.repo, cli.path);
}

fn app_arguments(arguments: &[OsString]) -> Option<Vec<OsString>> {
    let first = arguments.get(1)?.to_str()?;
    if first == JAYJAY_REVIEW_COMMAND || first == JAYJAY_TOOL_COMMAND {
        return Some(arguments[1..].to_vec());
    }
    if first == JAYJAY_CONFIG_COMMAND {
        return None;
    }
    if arguments.len() != 3
        || first.starts_with('-')
        || arguments
            .get(2)?
            .to_str()
            .is_none_or(|second| second.starts_with('-'))
    {
        return None;
    }
    let right = Path::new(arguments.get(2)?);
    let mode = if right.join("JJ-INSTRUCTIONS").is_file() {
        "edit"
    } else {
        "diff"
    };
    Some(vec![
        OsString::from(JAYJAY_TOOL_COMMAND),
        OsString::from(mode),
        arguments[1].clone(),
        arguments[2].clone(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forwards_app_owned_cli_commands() {
        for command in [JAYJAY_REVIEW_COMMAND, JAYJAY_TOOL_COMMAND] {
            let arguments = vec![OsString::from("jayjay"), OsString::from(command)];
            assert_eq!(
                app_arguments(&arguments),
                Some(vec![OsString::from(command)])
            );
        }
    }

    #[test]
    fn launcher_arguments_stay_in_rust() {
        for arg in [
            "--version",
            "-v",
            "--repo",
            ".",
            "--help",
            "config",
            "README.md",
        ] {
            let arguments = vec![OsString::from("jayjay"), OsString::from(arg)];
            assert!(app_arguments(&arguments).is_none());
        }
    }

    #[test]
    fn a_path_followed_by_a_flag_stays_in_rust() {
        for flag in ["-v", "--help", "--repo"] {
            let arguments = vec![
                OsString::from("jayjay"),
                OsString::from("."),
                OsString::from(flag),
            ];
            assert!(app_arguments(&arguments).is_none());
        }
    }

    #[test]
    fn two_paths_become_a_blocking_diff_tool_session() {
        let arguments = vec![
            OsString::from("jayjay"),
            OsString::from("/tmp/left"),
            OsString::from("/tmp/right"),
        ];
        assert_eq!(
            app_arguments(&arguments),
            Some(vec![
                OsString::from(JAYJAY_TOOL_COMMAND),
                OsString::from("diff"),
                OsString::from("/tmp/left"),
                OsString::from("/tmp/right"),
            ])
        );
    }

    #[test]
    fn config_arguments_never_become_a_diff_tool_session() {
        let arguments = vec![
            OsString::from("jayjay"),
            OsString::from(JAYJAY_CONFIG_COMMAND),
            OsString::from("extra"),
        ];
        assert!(app_arguments(&arguments).is_none());
    }

    #[test]
    fn jj_instructions_selects_diff_edit_mode() {
        let left = tempfile::tempdir().expect("left");
        let right = tempfile::tempdir().expect("right");
        std::fs::write(right.path().join("JJ-INSTRUCTIONS"), "instructions").expect("instructions");
        let arguments = vec![
            OsString::from("jayjay"),
            left.path().as_os_str().to_owned(),
            right.path().as_os_str().to_owned(),
        ];
        assert_eq!(app_arguments(&arguments).expect("forwarded")[1], "edit");
    }
}
