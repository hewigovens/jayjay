use std::sync::Arc;

use gpui::{App, AppContext, Global};

pub const GUIDE_URL: &str = "https://jayjay.hewig.dev/guide.html";
pub(crate) const FEEDBACK_ADDRESS: &str = "hi@hewig.dev";
pub const FEEDBACK_URL: &str = "mailto:hi@hewig.dev?subject=JayJay%20Feedback";

struct UrlOpener(Arc<dyn Fn(&str) -> bool + Send + Sync>);

impl Global for UrlOpener {}

impl Default for UrlOpener {
    fn default() -> Self {
        Self(Arc::new(crate::platform::open_url))
    }
}

pub fn install_url_opener(cx: &mut App, open_url: impl Fn(&str) -> bool + Send + Sync + 'static) {
    cx.set_global(UrlOpener(Arc::new(open_url)));
}

pub(crate) fn url_opener(cx: &mut App) -> Arc<dyn Fn(&str) -> bool + Send + Sync> {
    cx.default_global::<UrlOpener>().0.clone()
}

/// Hand `url` to the system handler off the UI thread; `xdg-open` can block until the handler exits.
pub(crate) fn open_url(cx: &mut App, url: &str) {
    let open_url = url_opener(cx);
    let url = url.to_owned();
    cx.background_spawn(async move {
        open_url(&url);
    })
    .detach();
}
