mod args;
mod launcher;
mod review;

use clap::Parser;

use crate::args::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    if cli.show_version {
        println!("jayjay {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if let Some(command) = cli.command {
        if let Err(error) = run_command(command) {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
        return;
    }

    launcher::launch_app(cli.repo, cli.path);
}

fn run_command(command: Commands) -> Result<(), String> {
    match command {
        Commands::Review { command } => review::run(command),
    }
}
