use std::path::Path;
use std::process::{Command, Output};

use super::Repo;
use super::environment;
use crate::types::*;

impl Repo {
    pub(crate) fn run_jj(&self, args: &[&str]) -> CoreResult<String> {
        let output = self.run_jj_output(args)?;
        self.checked_stdout(output)
    }

    pub(crate) fn run_jj_output(&self, args: &[&str]) -> CoreResult<Output> {
        let binary = environment::jj_binary();
        self.command_output(
            &binary,
            args,
            &format!("run jj {}", args.first().unwrap_or(&"")),
        )
    }

    pub(crate) fn run_jj_reload(&self, args: &[&str]) -> CoreResult<()> {
        self.run_jj(args)?;
        self.reload()
    }

    pub(crate) fn run_jj_quiet(&self, args: &[&str]) {
        let _ = self.run_jj_output(args);
    }

    pub(crate) fn command_output(
        &self,
        binary: &str,
        args: &[&str],
        context: &str,
    ) -> CoreResult<Output> {
        self.command_output_in(&self.path, binary, args, context)
    }

    pub(crate) fn command_output_in(
        &self,
        cwd: &Path,
        binary: &str,
        args: &[&str],
        context: &str,
    ) -> CoreResult<Output> {
        Command::new(binary)
            .current_dir(cwd)
            .args(args)
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("{context}: {e}"),
            })
    }

    pub(crate) fn ensure_success(&self, output: &Output, context: &str) -> CoreResult<()> {
        if output.status.success() {
            return Ok(());
        }
        let stderr = Self::stderr_text(output);
        let message = if stderr.is_empty() {
            context.to_owned()
        } else {
            format!("{context}: {stderr}")
        };
        Err(CoreError::Internal { message })
    }

    pub(crate) fn checked_stdout(&self, output: Output) -> CoreResult<String> {
        self.ensure_success(&output, "command failed")?;
        Ok(Self::stdout_text(&output))
    }

    pub(crate) fn stdout_text(output: &Output) -> String {
        String::from_utf8_lossy(&output.stdout).trim().to_string()
    }

    pub(crate) fn stderr_text(output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    }
}
