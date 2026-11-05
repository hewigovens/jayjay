use std::fmt::Write as _;
use std::path::Path;

use clap::{Parser, Subcommand};

use super::dispatch::describe_error;
use crate::repo::{Repo, WorkspacePrune};
use crate::{JayError, JayResult};

#[derive(Parser)]
#[command(disable_version_flag = true)]
pub(super) struct WorkspaceCli {
    #[command(subcommand)]
    pub(super) command: Option<WorkspaceCommand>,
}

#[derive(Subcommand)]
pub(super) enum WorkspaceCommand {
    /// Add a workspace named NAME at ../NAME
    Add {
        name: String,

        /// Revision the new working copy starts on
        #[arg(short, long, default_value = "trunk()", allow_hyphen_values = true)]
        revision: String,

        /// Path to the jj repository (default: current directory)
        #[arg(long, default_value = ".", allow_hyphen_values = true)]
        repo: String,
    },
    /// Forget and delete workspaces whose changes are all in trunk(); keep any with pending work
    Prune {
        /// Workspaces to prune
        #[arg(required_unless_present = "all", conflicts_with = "all")]
        names: Vec<String>,

        /// Prune every workspace except the current one
        #[arg(long)]
        all: bool,

        /// Path to the jj repository (default: current directory)
        #[arg(long, default_value = ".", allow_hyphen_values = true)]
        repo: String,
    },
}

impl WorkspaceCommand {
    pub(super) fn run(&self) -> JayResult<String> {
        match self {
            Self::Add {
                name,
                revision,
                repo,
            } => {
                let repo = Repo::open(Path::new(repo))?;
                let dest = repo.workspace_parent().join(name);
                let dest = dest.to_string_lossy();
                repo.workspace_add(&dest, name, revision)?;
                Ok(format!("Added workspace {name}: {dest}\n"))
            }
            Self::Prune { names, all, repo } => {
                let repo = Repo::open(Path::new(repo))?;
                let names = if *all {
                    repo.workspace_list()?
                        .into_iter()
                        .filter(|workspace| {
                            !workspace.is_current
                                && !repo.workspace_holds_store(Path::new(&workspace.path))
                        })
                        .map(|workspace| workspace.name)
                        .collect()
                } else {
                    names.clone()
                };
                let mut output = String::new();
                let mut failed = false;
                for name in &names {
                    let line = match repo.workspace_prune(name) {
                        Ok(WorkspacePrune::Removed {
                            path,
                            warning: None,
                        }) => {
                            format!("Removed {name}: {path}")
                        }
                        Ok(WorkspacePrune::Removed {
                            warning: Some(warning),
                            ..
                        }) => warning,
                        Ok(WorkspacePrune::Pending { changes }) => {
                            format!("Kept {name}: {changes} change(s) not in trunk()")
                        }
                        Ok(WorkspacePrune::Stale) => format!(
                            "Kept {name}: working copy is stale; run `jj workspace update-stale` there first"
                        ),
                        Err(error) => {
                            failed = true;
                            format!("Skipped {name}: {}", describe_error(&error))
                        }
                    };
                    let _ = writeln!(output, "{line}");
                }
                if failed {
                    return Err(JayError::internal(output.trim_end()));
                }
                Ok(output)
            }
        }
    }
}
