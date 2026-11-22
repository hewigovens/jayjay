use gpui::{AnyElement, App, InteractiveElement, IntoElement, ParentElement, Styled};

use super::RepoWindow;
use crate::app::theme::Theme;
use crate::ui::icons::glyph;
use crate::ui::notice_bar::{notice_bar, notice_bar_header};
use crate::ui::primitives::HelpTag;

pub(super) fn snapshot_warning_bar(view: &RepoWindow, t: &Theme, cx: &App) -> Option<AnyElement> {
    let error = view.vm.read(cx).background_snapshot_error.clone()?;
    Some(
        notice_bar(t.warning_bar_bg, t)
            .id("snapshot-warning-bar")
            .debug_selector(|| "snapshot-warning-bar".to_owned())
            .border_t_1()
            .child(notice_bar_header(
                glyph::WARNING,
                t.warning_fg,
                jayjay_i18n::repo::snapshot_out_of_date(),
            ))
            .help(error)
            .into_any_element(),
    )
}
