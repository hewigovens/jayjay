use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

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

    pub(in crate::repo) fn delayed_update(
        cx: &mut Context<Self>,
        delay: Duration,
        update: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) {
        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(delay).await;
            let _ = this.update(cx, update);
        })
        .detach();
    }

    pub(in crate::repo) fn repo_write_task(
        &mut self,
        cx: &mut Context<Self>,
        write: impl FnOnce(Arc<Repo>) -> CoreResult<()> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) -> Task<CoreResult<()>> {
        self.repo_result_task(cx, write, move |vm, _, cx| on_success(vm, cx))
    }

    pub(in crate::repo) fn repo_result_task<T>(
        &mut self,
        cx: &mut Context<Self>,
        read_or_write: impl FnOnce(Arc<Repo>) -> CoreResult<T> + Send + 'static,
        on_success: impl FnOnce(&mut Self, &T, &mut Context<Self>) + 'static,
    ) -> Task<CoreResult<T>>
    where
        T: Send + 'static,
    {
        let Some(repo) = self.repo.clone() else {
            self.present_error("repository is not open");
            cx.notify();
            return cx.spawn(async move |_, _| Err(Error::internal("repository is not open")));
        };

        self.clear_error();
        // Stamp before the write so the FS echo from our own jj mutation is ignored.
        self.last_internal_mutation_at = Some(std::time::Instant::now());
        self.begin_refreshing(cx);

        cx.spawn(async move |this, cx| {
            let result = cx
                .background_spawn(async move { read_or_write(repo) })
                .await;
            this.update(cx, move |vm, cx| {
                vm.finish_refreshing(cx);
                match result {
                    Ok(value) => {
                        on_success(vm, &value, cx);
                        Ok(value)
                    }
                    Err(error) => {
                        vm.present_error(&error);
                        cx.notify();
                        Err(error)
                    }
                }
            })
            .unwrap_or_else(|error| Err(Error::internal(error)))
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
