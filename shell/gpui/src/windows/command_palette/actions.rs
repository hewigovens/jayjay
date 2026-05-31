use gpui::{App, Entity};

use crate::app::config::{self, AppearanceMode};
use crate::app::tools;
use crate::log::LogView;
use crate::ui::icons::glyph;
use crate::windows::settings::SettingsView;

/// Context passed to a palette action's dispatcher.
pub(super) struct PaletteCtx<'a> {
    pub repo_path: &'a str,
    pub log_view: Option<Entity<LogView>>,
}

/// One row in the palette: human label, search keywords, and a side-effect.
pub(super) struct PaletteAction {
    pub name: &'static str,
    pub keywords: &'static [&'static str],
    pub glyph_str: &'static str,
    pub dispatch: fn(&PaletteCtx, &mut App),
}

pub(super) const ACTIONS: &[PaletteAction] = &[
    PaletteAction {
        name: "Open Settings",
        keywords: &["settings", "preferences", "config"],
        glyph_str: glyph::GEAR,
        dispatch: |_, cx| SettingsView::open(cx),
    },
    PaletteAction {
        name: "Open Bookmark Manager",
        keywords: &["bookmark", "bookmarks", "manager", "branch", "branches"],
        glyph_str: glyph::GIT_BRANCH,
        dispatch: |ctx, cx| {
            if let Some(view) = ctx.log_view.clone() {
                view.update(cx, |view, cx| view.open_bookmark_manager(cx));
            }
        },
    },
    PaletteAction {
        name: "Open in Terminal",
        keywords: &["terminal", "shell", "open"],
        glyph_str: glyph::ARROW_CIRCLE_RIGHT,
        dispatch: |ctx, cx| {
            tools::open_in_terminal(ctx.repo_path, cx);
        },
    },
    PaletteAction {
        name: "Theme: System",
        keywords: &["theme", "appearance", "system"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::System),
    },
    PaletteAction {
        name: "Theme: Light",
        keywords: &["theme", "appearance", "light"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Light),
    },
    PaletteAction {
        name: "Theme: Dark",
        keywords: &["theme", "appearance", "dark"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| set_appearance(cx, AppearanceMode::Dark),
    },
    PaletteAction {
        name: "Toggle Side-by-side Diff",
        keywords: &["diff", "split", "side", "by", "unified"],
        glyph_str: glyph::COLUMNS,
        dispatch: |_, cx| config::update(cx, |c| c.diff.side_by_side ^= true),
    },
    PaletteAction {
        name: "Toggle Ignore Whitespace",
        keywords: &["whitespace", "diff", "ignore"],
        glyph_str: glyph::WHITESPACE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.ignore_whitespace ^= true),
    },
    PaletteAction {
        name: "Toggle Tree File List",
        keywords: &["tree", "file", "folder", "list"],
        glyph_str: glyph::FOLDER_SIMPLE,
        dispatch: |_, cx| config::update(cx, |c| c.diff.tree_file_list ^= true),
    },
];

fn set_appearance(cx: &mut App, mode: AppearanceMode) {
    config::update(cx, move |c| c.appearance = mode);
}
