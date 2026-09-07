use std::path::PathBuf;

use jayjay_core::external_tools::{ExternalToolInvocation, parse_external_tool_invocation};
use jayjay_core::{CoreResult, JAYJAY_CONFIG_COMMAND, JAYJAY_REVIEW_COMMAND, JAYJAY_TOOL_COMMAND};

use super::arguments::{Arguments, Command};

#[derive(Debug, PartialEq)]
pub(crate) enum GuiLaunch {
    Repository {
        path: Option<PathBuf>,
        foreground: bool,
    },
    ExternalTool(ExternalToolInvocation),
}

#[derive(Debug, PartialEq)]
pub(super) enum Invocation {
    Command(Vec<String>),
    Gui(GuiLaunch),
}

impl TryFrom<Arguments> for Invocation {
    type Error = jayjay_core::CoreError;

    fn try_from(arguments: Arguments) -> CoreResult<Self> {
        let forwarded = if let Some(command) = arguments.command {
            let (name, mut forwarded) = match command {
                Command::Config(args) => (JAYJAY_CONFIG_COMMAND, args.arguments),
                Command::Review(args) => (JAYJAY_REVIEW_COMMAND, args.arguments),
                Command::Tool(args) => (JAYJAY_TOOL_COMMAND, args.arguments),
            };
            forwarded.insert(0, name.to_owned());
            if name != JAYJAY_TOOL_COMMAND {
                return Ok(Self::Command(forwarded));
            }
            forwarded
        } else if arguments.paths.len() == 2 {
            arguments.paths
        } else {
            return Ok(Self::Gui(GuiLaunch::Repository {
                path: arguments
                    .repo
                    .or_else(|| arguments.paths.into_iter().next().map(PathBuf::from)),
                foreground: arguments.foreground,
            }));
        };
        parse_external_tool_invocation(&forwarded).map(|invocation| {
            Self::Gui(GuiLaunch::ExternalTool(
                invocation.expect("tool or two paths select an external session"),
            ))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse(args: &[&str]) -> Invocation {
        let arguments =
            Arguments::try_parse_from(std::iter::once("jayjay").chain(args.iter().copied()))
                .unwrap();
        Invocation::try_from(arguments).unwrap()
    }

    #[test]
    fn repository_options_produce_a_path_and_launch_policy() {
        for args in [&[][..], &["."], &["--repo", "."]] {
            let Invocation::Gui(request) = parse(args) else {
                panic!("expected GUI")
            };
            assert_eq!(
                request,
                GuiLaunch::Repository {
                    path: (!args.is_empty()).then(|| PathBuf::from(".")),
                    foreground: false
                }
            );
        }
        let Invocation::Gui(request) = parse(&["--foreground", "--", "--help"]) else {
            panic!("expected GUI")
        };
        assert_eq!(
            request,
            GuiLaunch::Repository {
                path: Some(PathBuf::from("--help")),
                foreground: true
            }
        );
    }

    #[test]
    fn explicit_and_two_path_external_tools_stay_in_the_foreground() {
        for args in [
            &["tool", "merge", "left", "base", "right", "output"][..],
            &["--foreground", "left", "right"],
        ] {
            let Invocation::Gui(request) = parse(args) else {
                panic!("expected GUI")
            };
            assert!(matches!(request, GuiLaunch::ExternalTool(_)));
        }
    }

    #[test]
    fn shared_commands_keep_their_arguments() {
        for args in [
            &["review", "notes", "--repo", ".", "--format", "json"][..],
            &["config", "extra"],
        ] {
            assert_eq!(
                parse(args),
                Invocation::Command(args.iter().map(|arg| (*arg).to_owned()).collect())
            );
        }
    }
}
