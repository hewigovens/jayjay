use gpui::rgba;

use crate::app::theme::Theme;

pub(crate) fn selection_bg(theme: &Theme) -> gpui::Rgba {
    rgba(((theme.selected_bg as u64) << 8) as u32 | 0x66)
}
