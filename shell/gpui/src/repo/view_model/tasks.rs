use std::future::Future;
use std::sync::Arc;

use gpui::{AppContext, Context, Task};
use jayjay_core::{CoreResult, Error, Repo};

use super::RepoViewModel;

impl RepoViewModel {
    pub(in crate::repo) fn background_update<T>(
        cx: &mut Context<Self>,
        future: impl Future<Output = T> + Send + 'static,
        update: impl FnOnce(&mut Self, T, &mut Context<Self>) + 'static,
    ) where
        T: Send + 'static,
    {
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(future).await;
            let _ = this.update(cx, move |vm, cx| update(vm, result, cx));
        })
        .detach();
    }

    pub(in crate::repo) fn repo_write_task(
        &mut self,
        cx: &mut Context<Self>,
        write: impl FnOnce(Arc<Repo>) -> CoreResult<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) -> Task<CoreResult<()>> {
        let Some(repo) = self.repo.clone() else {
            self.present_error("repository is not open");
            cx.notify();
            return cx.spawn(async move |_, _| Err(Error::internal("repository is not open")));
        };

        self.loading.refreshing = true;
        self.clear_error();
        cx.notify();

        Self::core_result_task(cx, async move { write(repo) }, move |vm, result, cx| {
            vm.loading.refreshing = false;
            match result {
                Ok(()) => {
                    on_success(vm, cx);
                    Ok(())
                }
                Err(error) => {
                    vm.present_error(&error);
                    cx.notify();
                    Err(error)
                }
            }
        })
    }

    pub(in crate::repo) fn core_result_task(
        cx: &mut Context<Self>,
        future: impl Future<Output = CoreResult<()>> + Send + 'static,
        update: impl FnOnce(&mut Self, CoreResult<()>, &mut Context<Self>) -> CoreResult<()> + 'static,
    ) -> Task<CoreResult<()>> {
        cx.spawn(async move |this, cx| {
            let result = cx.background_spawn(future).await;
            this.update(cx, move |vm, cx| update(vm, result, cx))
                .unwrap_or_else(|error| Err(Error::internal(error)))
        })
    }
}
