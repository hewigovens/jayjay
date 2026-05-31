use std::path::PathBuf;

use gpui::Context;

use super::state::{CommandOutput, CommandPalette};

impl CommandPalette {
    pub(super) fn run_command(&mut self, body: String, cx: &mut Context<Self>) {
        if body.is_empty() {
            return;
        }
        let display = format!("jj {body}");
        if jayjay_core::JjCommand::new(&body).parse_args().is_none() {
            self.output = CommandOutput::Done {
                display,
                output: "Unclosed quote in jj command.".to_owned(),
                exit_code: -1,
            };
            cx.notify();
            return;
        }
        self.output = CommandOutput::Running {
            display: display.clone(),
        };
        cx.notify();
        let cwd = self.repo_path.to_string();
        let log_view = self.log_view.clone();
        let command_for_history = body.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { execute(body, &cwd, display) })
                .await;
            let success = result.is_success();
            let _ = this.update(cx, |this, cx| {
                this.record_command_history(&command_for_history);
                this.output = result;
                cx.notify();
            });
            if success && let Some(log_view) = log_view {
                log_view.update(cx, |view, cx| {
                    let vm = view.vm.clone();
                    vm.update(cx, |vm, cx| vm.refresh(false, cx));
                });
            }
        })
        .detach();
    }
}

fn execute(body: String, cwd: &str, display: String) -> CommandOutput {
    match jayjay_core::JjCommand::new(body).run_in_path(&PathBuf::from(cwd)) {
        Ok(result) => CommandOutput::Done {
            display,
            output: result.output,
            exit_code: result.exit_code,
        },
        Err(e) => CommandOutput::Done {
            display,
            output: format!("{e}"),
            exit_code: -1,
        },
    }
}
