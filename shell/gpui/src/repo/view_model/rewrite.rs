use std::sync::Arc;

use gpui::{Context, Task};
use jayjay_core::{JayResult, Repo, capture_mutation_operation_id};

use super::RepoViewModel;

pub(crate) struct Rewrite<T> {
    pub(crate) value: T,
    pub(crate) operation_id: Option<String>,
}

impl RepoViewModel {
    pub(crate) fn repo_rewrite_task<T>(
        &mut self,
        cx: &mut Context<Self>,
        write: impl FnOnce(Arc<Repo>) -> JayResult<T> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &T, &mut Context<Self>) + 'static,
    ) -> Task<JayResult<Rewrite<T>>>
    where
        T: Send + 'static,
    {
        self.repo_result_task(
            cx,
            move |repo| {
                let (value, operation_id) = capture_mutation_operation_id(|| write(repo));
                Ok(Rewrite {
                    value: value?,
                    operation_id,
                })
            },
            move |vm, rewrite, cx| on_success(vm, &rewrite.value, cx),
        )
    }
}
