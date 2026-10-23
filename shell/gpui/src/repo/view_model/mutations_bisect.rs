use std::sync::Arc;

use gpui::{Context, Task};
use jayjay_core::{BisectPhase, BisectState, BisectVerdict, JayResult, Repo};

use super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn start_bisect(&mut self, range: String, cx: &mut Context<Self>) {
        self.bisect_transition(cx, move |repo| repo.start_bisect(&range), show_bisect);
    }

    pub(crate) fn start_bisect_between(
        &mut self,
        first: String,
        second: String,
        cx: &mut Context<Self>,
    ) {
        self.bisect_transition(
            cx,
            move |repo| repo.start_bisect_between(&first, &second),
            show_bisect,
        );
    }

    pub(crate) fn mark_bisect(
        &mut self,
        commit_id: String,
        verdict: BisectVerdict,
        cx: &mut Context<Self>,
    ) {
        self.bisect_transition(
            cx,
            move |repo| repo.bisect_mark(&commit_id, verdict),
            show_bisect,
        );
    }

    pub(crate) fn restart_bisect(&mut self, cx: &mut Context<Self>) {
        self.bisect_transition(cx, |repo| repo.bisect_restart(), show_bisect);
    }

    pub(crate) fn dismiss_bisect(&mut self, cx: &mut Context<Self>) {
        self.bisect_transition(
            cx,
            |repo| repo.bisect_dismiss(),
            |vm, _, cx| {
                vm.bisect.state = None;
                cx.notify();
            },
        );
    }

    pub(crate) fn stop_bisect(&mut self, cx: &mut Context<Self>) {
        if !self.bisect.can_stop() {
            return;
        }
        if let Some(sync) = self.bisect.begin_stop() {
            sync.cancel();
        }
        let task = self.repo_result_task_without_indicator(
            cx,
            |repo| repo.bisect_stop(),
            |vm, _, cx| {
                vm.bisect.finish_activity();
                vm.bisect.state = None;
                vm.refresh(false, cx);
            },
        );
        release_bisect_on_failure(task, cx);
    }

    pub(crate) fn run_bisect(&mut self, command: String, to_end: bool, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else {
            self.present_error(jayjay_i18n::repo::repository_not_open());
            cx.notify();
            return;
        };
        let sync = repo.sync_token();
        if !self.bisect.begin_command(sync.clone()) {
            return;
        }
        let task = self.repo_result_task_without_indicator(
            cx,
            move |_| {
                if to_end {
                    repo.bisect_run_to_end(&command, &sync)
                } else {
                    repo.bisect_run_step(&command, &sync)
                }
            },
            |vm, state, cx| {
                if vm.bisect.is_stopping() {
                    return;
                }
                vm.bisect.finish_activity();
                match state {
                    Some(state) => show_bisect(vm, state, cx),
                    None => {
                        vm.bisect.state = None;
                        vm.refresh(false, cx);
                    }
                }
            },
        );
        cx.spawn(async move |this, cx| {
            if task.await.is_err() {
                let _ = this.update(cx, |vm, cx| {
                    if !vm.bisect.is_stopping() {
                        vm.bisect.finish_activity();
                    }
                    cx.notify();
                });
            }
        })
        .detach();
    }

    pub(crate) fn stop_bisect_for_shutdown(&mut self) {
        let Some(repo) = &self.repo else {
            return;
        };
        if let Some(sync) = self.bisect.begin_stop() {
            sync.cancel();
        }
        let _ = repo.shutdown_bisect();
        self.bisect.state = None;
        self.bisect.finish_activity();
    }

    fn bisect_transition<T: Send + 'static>(
        &mut self,
        cx: &mut Context<Self>,
        op: impl FnOnce(Arc<Repo>) -> JayResult<T> + Send + 'static,
        on_ok: impl FnOnce(&mut Self, &T, &mut Context<Self>) + 'static,
    ) {
        if !self.bisect.begin_transition() {
            return;
        }
        let task = self.repo_result_task_without_indicator(cx, op, move |vm, value, cx| {
            vm.bisect.finish_activity();
            on_ok(vm, value, cx);
        });
        release_bisect_on_failure(task, cx);
    }
}

fn release_bisect_on_failure<T: 'static>(
    task: Task<JayResult<T>>,
    cx: &mut Context<RepoViewModel>,
) {
    cx.spawn(async move |this, cx| {
        if task.await.is_err() {
            let _ = this.update(cx, |vm, cx| {
                vm.bisect.finish_activity();
                cx.notify();
            });
        }
    })
    .detach();
}

fn show_bisect(vm: &mut RepoViewModel, state: &BisectState, cx: &mut Context<RepoViewModel>) {
    vm.bisect.state = Some(state.clone());
    vm.refresh_preferring(false, bisect_refresh_target(state), cx);
}

fn bisect_refresh_target(state: &BisectState) -> Option<(String, String)> {
    if state.phase == BisectPhase::Finished {
        let first_bad = state.result.as_ref()?.bad.first()?;
        return Some((first_bad.change_id.clone(), first_bad.commit_id.clone()));
    }
    state
        .candidate
        .as_ref()
        .map(|candidate| (candidate.change_id.clone(), candidate.commit_id.clone()))
}
