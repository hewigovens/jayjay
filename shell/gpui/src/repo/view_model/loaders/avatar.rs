use gpui::Context;

use super::super::RepoViewModel;

impl RepoViewModel {
    pub(crate) fn ensure_avatar(&mut self, email: String, cx: &mut Context<Self>) {
        if email.trim().is_empty() {
            return;
        }
        if self.avatar_in_flight.contains(&email) {
            return;
        }
        if let Some(path) = crate::ui::avatar::cache_path(&email)
            && path.exists()
        {
            return;
        }
        self.avatar_in_flight.insert(email.clone());
        let email_for_remove = email.clone();
        Self::background_update(
            cx,
            async move {
                crate::ui::avatar::fetch_blocking(&email);
            },
            move |vm, (), cx| {
                vm.avatar_in_flight.remove(&email_for_remove);
                cx.notify();
            },
        );
    }
}
