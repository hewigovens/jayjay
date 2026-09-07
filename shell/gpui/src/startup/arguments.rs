use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "jayjay",
    version,
    about = "Native desktop client for Jujutsu",
    disable_version_flag = true,
    after_help = "With no path, open the repository containing the current directory, the most recent repository, or the repository list.\nTwo paths open a blocking diff session; jj's JJ-INSTRUCTIONS file selects edit mode.\nOn Linux, ordinary GUI launches detach from the terminal."
)]
pub(super) struct Arguments {
    #[arg(short = 'v', visible_short_alias = 'V', long = "version", action = clap::ArgAction::Version, help = "Print version")]
    _version: Option<bool>,

    #[arg(long, help = "Keep the GUI and its logs attached to the terminal")]
    pub foreground: bool,

    #[arg(
        short,
        long,
        value_name = "PATH",
        conflicts_with = "paths",
        help = "Open a jj repository at PATH"
    )]
    pub repo: Option<PathBuf>,

    #[arg(value_name = "PATH", num_args = 0..=2, help = "Repository path, or two paths to compare")]
    pub paths: Vec<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub(super) enum Command {
    #[command(about = "Print jj configuration for JayJay's diff, edit, and merge tools")]
    Config(ForwardedArguments),
    #[command(about = "Manage review notes (notes, add-note, resolve-note)")]
    Review(ForwardedArguments),
    #[command(
        about = "Open a blocking diff, edit, or merge session",
        after_help = "Usage:\n  jayjay tool diff <LEFT> <RIGHT>\n  jayjay tool edit <LEFT> <RIGHT>\n  jayjay tool merge <LEFT> <BASE> <RIGHT> <OUTPUT> [<PATH> <MARKER_LENGTH>]"
    )]
    Tool(ForwardedArguments),
}

#[derive(Args)]
pub(super) struct ForwardedArguments {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub arguments: Vec<String>,
}
