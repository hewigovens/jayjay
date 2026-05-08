use std::path::Path;

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
    match jayjay_core::run_jj_command_in_path(Path::new(cwd), &body) {
        Ok(out) => CommandOutput::Done {
            display: out.display,
            stdout: out.stdout,
            stderr: out.stderr,
            success: out.success,
        },
        Err(e) => CommandOutput::Done {
            display,
            stdout: String::new(),
            stderr: e.to_string(),
            success: false,
        },
    }
}
