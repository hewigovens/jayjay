mod platform;

use std::ffi::OsString;
use std::process::Command;

use super::{app, gpui};

// Falls back to the GPUI binary when no macOS app bundle is found — the only option on Linux, and useful on dev checkouts that only built the GPUI shell.
pub(crate) fn exec_app<I>(arguments: I) -> !
where
    I: IntoIterator<Item = OsString>,
{
    let (executable, is_macos_app) = if let Some(executable) = app::find_app_executable() {
        (executable, true)
    } else if let Some(executable) = gpui::find_gpui_executable() {
        (executable, false)
    } else {
        eprintln!("error: JayJay executable not found");
        eprintln!("Install JayJay.app, or build the GPUI shell with: just gpui");
        std::process::exit(127);
    };

    let arguments = executable_arguments(arguments, is_macos_app);
    let mut command = Command::new(&executable);
    command.args(arguments);
    let error = platform::exec(command);

    eprintln!("error: failed to run {}: {error}", executable.display());
    std::process::exit(127);
}

fn executable_arguments<I>(arguments: I, is_macos_app: bool) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut arguments: Vec<_> = arguments.into_iter().collect();
    if is_macos_app && arguments.first().is_some_and(|argument| argument == "tool") {
        arguments.splice(
            0..0,
            [
                OsString::from("-ApplePersistenceIgnoreState"),
                OsString::from("YES"),
            ],
        );
    }
    arguments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn macos_external_tools_ignore_restored_windows() {
        assert_eq!(
            executable_arguments(
                [
                    OsString::from("tool"),
                    OsString::from("merge"),
                    OsString::from("left"),
                ],
                true,
            ),
            [
                OsString::from("-ApplePersistenceIgnoreState"),
                OsString::from("YES"),
                OsString::from("tool"),
                OsString::from("merge"),
                OsString::from("left"),
            ]
        );
    }

    #[test]
    fn macos_two_path_sessions_ignore_restored_windows_after_forwarding() {
        let original = [
            OsString::from("jayjay"),
            OsString::from("left"),
            OsString::from("right"),
        ];
        let forwarded = crate::app_arguments(&original).expect("two paths are forwarded");

        assert_eq!(
            executable_arguments(forwarded, true),
            [
                OsString::from("-ApplePersistenceIgnoreState"),
                OsString::from("YES"),
                OsString::from("tool"),
                OsString::from("diff"),
                OsString::from("left"),
                OsString::from("right"),
            ]
        );
    }

    #[test]
    fn gpui_external_tools_do_not_receive_cocoa_arguments() {
        assert_eq!(
            executable_arguments([OsString::from("tool"), OsString::from("diff")], false,),
            [OsString::from("tool"), OsString::from("diff")]
        );
    }
}
