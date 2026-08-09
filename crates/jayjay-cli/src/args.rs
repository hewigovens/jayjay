use clap::{Parser, Subcommand};

/// Native GUI for Jujutsu version control
#[derive(Parser)]
#[command(name = "jayjay", version, about)]
#[command(disable_version_flag = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,

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
pub(crate) enum Command {
    /// Print jj configuration for using JayJay as a diff, edit, and merge tool
    Config,
}
