use std::sync::Arc;

use gpui::Context;
use jayjay_core::Repo;

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
        let repo_window = self.repo_window.clone();
        let repo = repo_window
            .as_ref()
            .and_then(|window| window.read(cx).view_model().read(cx).repo.clone());
        let command_for_history = body.clone();
        cx.spawn(async move |this, cx| {
            let result = cx
                .background_executor()
                .spawn(async move { execute(body, repo, &cwd, display) })
                .await;
            let success = result.is_success();
            let _ = this.update(cx, |this, cx| {
                this.record_command_history(&command_for_history);
                this.output = result;
                cx.notify();
            });
            if success && let Some(repo_window) = repo_window {
                repo_window.update(cx, |view, cx| {
                    let vm = view.vm.clone();
                    vm.update(cx, |vm, cx| vm.refresh(false, cx));
                });
            }
        })
        .detach();
    }
}

fn execute(body: String, repo: Option<Arc<Repo>>, cwd: &str, display: String) -> CommandOutput {
    let command = jayjay_core::JjCommand::new(body);
    let result = match &repo {
        Some(repo) => command.run_in_repo(repo),
        None => command.run_in_path(std::path::Path::new(cwd)),
    };
    match result {
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
