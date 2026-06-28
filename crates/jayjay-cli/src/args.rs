use clap::{Parser, Subcommand, ValueEnum};

/// Native GUI for Jujutsu version control
#[derive(Parser)]
#[command(name = "jayjay", version, about)]
#[command(disable_version_flag = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Commands>,

    /// Path to a jj repository (default: current directory if it contains .jj)
    pub(crate) path: Option<String>,

    /// Open repository at PATH
    #[arg(short, long)]
    pub(crate) repo: Option<String>,

    /// Print version
    #[arg(short = 'v', long = "version")]
    pub(crate) show_version: bool,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Review-note commands for agent workflows
    Review {
        #[command(subcommand)]
        command: ReviewCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum ReviewCommand {
    /// List review notes for the current working-copy change
    Notes {
        /// Path to a jj repository
        #[arg(long, default_value = ".")]
        repo: String,

        /// Output format
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Include resolved notes
        #[arg(long)]
        include_resolved: bool,
    },
    /// Resolve a review note by id
    ResolveNote {
        /// Stable note id
        id: String,

        /// Path to a jj repository
        #[arg(long, default_value = ".")]
        repo: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum OutputFormat {
    Text,
    Json,
}
