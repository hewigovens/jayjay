use std::process::{Command, ExitStatus, Output};

use jj_lib::bisect::Evaluation;
use jj_lib::object_id::ObjectId as _;

use super::super::command_process::SyncToken;
use super::session::Session;
use crate::Repo;
use crate::types::*;

impl Repo {
    pub fn bisect_run_step(
        &self,
        command: &str,
        sync: &SyncToken,
    ) -> JayResult<Option<BisectState>> {
        if command.trim().is_empty() {
            return Err(JayError::refused("Enter a command to run."));
        }
        if sync.check().is_err() {
            self.bisect_stop()?;
            return Ok(None);
        }
        let Some(commit_id) = self.candidate_commit_id()? else {
            return Ok(self.bisect_state());
        };
        let output = self.run_bisect_command(command, &commit_id, sync);
        // The cancel latch decides, not the signal or the exit code: a killed test is Stop, never Bad.
        if sync.check().is_err() {
            self.bisect_stop()?;
            return Ok(None);
        }
        match output {
            Ok(output) => self.record_command_and_advance(&commit_id, output),
            Err(JayError::Canceled) => {
                self.bisect_stop()?;
                Ok(None)
            }
            Err(error) => self.note_command_error(error.to_string()),
        }
    }

    pub fn bisect_run_to_end(
        &self,
        command: &str,
        sync: &SyncToken,
    ) -> JayResult<Option<BisectState>> {
        loop {
            match self.bisect_run_step(command, sync)? {
                None => return Ok(None),
                Some(state)
                    if state.phase != BisectPhase::Evaluating || state.command_error.is_some() =>
                {
                    return Ok(Some(state));
                }
                Some(_) => {}
            }
        }
    }

    fn candidate_commit_id(&self) -> JayResult<Option<String>> {
        let _write = self.write_guard()?;
        let Some(session) = self.bisect_lock().clone() else {
            return Err(JayError::refused("No bisect is running."));
        };
        if session.phase() != BisectPhase::Evaluating {
            return Err(JayError::refused("There is no candidate to test."));
        }
        if !self.range_intact(&session)? {
            if let Some(session) = self.bisect_lock().as_mut() {
                session.pause();
            }
            return Ok(None);
        }
        self.ensure_candidate_checkout(&session)?;
        Ok(session.candidate.as_ref().map(|commit| commit.id().hex()))
    }

    fn record_command_and_advance(
        &self,
        commit_id: &str,
        output: Output,
    ) -> JayResult<Option<BisectState>> {
        let _write = self.write_guard()?;
        let Some(mut session) = self.bisect_lock().clone() else {
            return Ok(None);
        };
        let result = self.apply_command_output(&mut session, commit_id, output);
        *self.bisect_lock() = Some(session);
        result
    }

    fn apply_command_output(
        &self,
        session: &mut Session,
        commit_id: &str,
        output: Output,
    ) -> JayResult<Option<BisectState>> {
        if session.phase() != BisectPhase::Evaluating {
            return Ok(Some(session.state()));
        }
        let id = match session.guard_candidate(commit_id) {
            Ok(id) => id,
            Err(_) => return Ok(Some(session.state())),
        };
        if !self.range_intact(session)? {
            session.pause();
            return Ok(Some(session.state()));
        }
        self.ensure_candidate_checkout(session)?;
        session.set_command_output(output_tail(&output));
        session.push_mark(id, evaluation_from_status(output.status));
        self.advance_after_mark(session).map(Some)
    }

    fn note_command_error(&self, message: String) -> JayResult<Option<BisectState>> {
        let _write = self.write_guard()?;
        let mut guard = self.bisect_lock();
        let Some(session) = guard.as_mut() else {
            return Ok(None);
        };
        session.set_command_error(message);
        Ok(Some(session.state()))
    }

    fn run_bisect_command(
        &self,
        line: &str,
        commit_hex: &str,
        sync: &SyncToken,
    ) -> JayResult<Output> {
        let _running = self.bisect_execution_lock();
        let mut command = shell_command(line);
        command
            .current_dir(&self.path)
            .env("JJ_BISECT_TARGET", commit_hex)
            .env("JJ_EXECUTABLE_PATH", super::super::environment::jj_binary());
        sync.output(&mut command, "bisect command")
    }
}

fn evaluation_from_status(status: ExitStatus) -> Evaluation {
    match status.code() {
        Some(0) => Evaluation::Good,
        Some(125) => Evaluation::Skip,
        Some(127) => Evaluation::Abort,
        _ => Evaluation::Bad,
    }
}

fn output_tail(output: &Output) -> String {
    let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
    let err = String::from_utf8_lossy(&output.stderr);
    if !err.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(&err);
    }
    let lines: Vec<&str> = text.lines().collect();
    lines
        .iter()
        .skip(lines.len().saturating_sub(20))
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
}

fn shell_command(line: &str) -> Command {
    #[cfg(windows)]
    let (program, flag) = ("cmd", "/C");
    #[cfg(not(windows))]
    let (program, flag) = ("/bin/sh", "-c");
    let mut command = super::super::environment::command(program);
    command.arg(flag);
    #[cfg(windows)]
    std::os::windows::process::CommandExt::raw_arg(&mut command, line);
    #[cfg(not(windows))]
    command.arg(line);
    command
}
