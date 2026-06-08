use std::time::Duration;

use gpui::Context;

use super::RepoViewModel;

const MIN_REFRESH_INDICATOR: Duration = Duration::from_secs(1);

impl RepoViewModel {
    pub(in crate::repo) fn begin_refreshing(&mut self, cx: &mut Context<Self>) {
        self.loading.refreshing = true;
        self.loading.refresh_indicator = true;
        self.loading.refresh_minimum_elapsed = false;
        self.loading.refresh_indicator_gen = self.loading.refresh_indicator_gen.wrapping_add(1);
        let generation = self.loading.refresh_indicator_gen;
        Self::delayed_update(cx, MIN_REFRESH_INDICATOR, move |vm, cx| {
            vm.refresh_indicator_minimum_elapsed(generation, cx);
        });
        cx.notify();
    }

    pub(in crate::repo) fn finish_refreshing(&mut self, cx: &mut Context<Self>) {
        self.loading.refreshing = false;
        if self.loading.refresh_minimum_elapsed {
            self.loading.refresh_indicator = false;
        }
        cx.notify();
    }

    fn refresh_indicator_minimum_elapsed(&mut self, generation: u64, cx: &mut Context<Self>) {
        if self.loading.refresh_indicator_gen != generation {
            return;
        }
        self.loading.refresh_minimum_elapsed = true;
        if !self.loading.refreshing {
            self.loading.refresh_indicator = false;
        }
        cx.notify();
    }
}
