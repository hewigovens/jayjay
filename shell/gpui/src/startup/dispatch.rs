use clap::Parser;

use super::arguments::Arguments;
use super::invocation::Invocation;

pub fn run(arguments: &[String]) -> i32 {
    let arguments = match Arguments::try_parse_from(
        std::iter::once("jayjay").chain(arguments.iter().map(String::as_str)),
    ) {
        Ok(arguments) => arguments,
        Err(error) => {
            let _ = error.print();
            return error.exit_code();
        }
    };
    let invocation = match Invocation::try_from(arguments) {
        Ok(invocation) => invocation,
        Err(error) => {
            eprintln!("error: {error}");
            return 1;
        }
    };
    match invocation {
        Invocation::Command(arguments) => {
            let outcome = jayjay_core::run_app_cli_command(&arguments, env!("CARGO_PKG_VERSION"))
                .expect("startup forwards only shared CLI commands");
            if outcome.is_error() {
                eprint!("{}", outcome.message);
            } else {
                print!("{}", outcome.message);
            }
            outcome.exit_code
        }
        Invocation::Gui(launch) => {
            #[cfg(target_os = "linux")]
            if let super::GuiLaunch::Repository {
                path,
                foreground: false,
            } = &launch
            {
                if let Err(error) = super::linux::detach(path.as_deref()) {
                    eprintln!("error: failed to launch JayJay: {error}");
                    return 1;
                }
                return 0;
            }
            crate::app::runtime::run(launch);
            0
        }
    }
}
