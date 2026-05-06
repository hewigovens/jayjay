use std::process::Command;

use gpui::Context;

use super::state::{CommandOutput, CommandPalette};

impl CommandPalette {
    pub(super) fn run_command(&mut self, body: String, cx: &mut Context<Self>) {
        if body.is_empty() {
            return;
        }
        let display = format!("jj {body}");
        self.output = CommandOutput::Running {
            display: display.clone(),
        };
        cx.notify();
        let cwd = self.repo_path.to_string();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { execute(body, &cwd, display) })
                .await;
            let _ = this.update(cx, |this, cx| {
                this.output = result;
                cx.notify();
            });
        })
        .detach();
    }
}

fn execute(body: String, cwd: &str, display: String) -> CommandOutput {
    let binary = jayjay_core::jj_binary();
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c")
        .arg(format!("{binary} {body}"))
        .current_dir(cwd);
    match cmd.output() {
        Ok(out) => CommandOutput::Done {
            display,
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
            success: out.status.success(),
        },
        Err(e) => CommandOutput::Done {
            display,
            stdout: String::new(),
            stderr: format!("failed to spawn: {e}"),
            success: false,
        },
    }
}
