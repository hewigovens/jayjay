use std::path::{Path, PathBuf};

use gpui::{App, Menu, MenuItem, PathPromptOptions};

use super::actions::{
    ClearRecentRepositories, NewWorkspace, OpenAbout, OpenBookmarkManager, OpenCommandPalette,
    OpenFind, OpenJujutsuDocumentation, OpenOperationLog, OpenRecentRepository,
    OpenRemoteRepository, OpenRepoInEditor, OpenRepoInTerminal, OpenRepository, OpenSettings,
    OpenUserGuide, Quit, ReportIssue, ResetZoom, SendFeedback, ShowRepoInFileManager,
    ToggleHideGitLfsFiles, ToggleIgnoreWhitespace, ToggleSideBySideDiff, ToggleTreeFileList,
    ZoomIn, ZoomOut,
};
use super::config::{self, current};
use super::tools;
use crate::app::links::GUIDE_URL;
use crate::windows::settings::{SettingsSection, SettingsView};

const JUJUTSU_DOCS_URL: &str = "https://jj-vcs.github.io/jj/latest/";
const REPORT_ISSUE_URL: &str = "https://github.com/hewigovens/jayjay/issues";

pub fn install(cx: &mut App) {
    register_global_actions(cx);
    refresh(cx);
}

pub(crate) fn refresh(cx: &mut App) {
    let menus = app_menus(cx);
    cx.set_menus(menus);
}

fn app_menus(cx: &mut App) -> Vec<Menu> {
    let cfg = current(cx);
    vec![
        app_menu(),
        file_menu(&cfg.recent_repos),
        Menu::new("Edit").items([MenuItem::action("Find...", OpenFind)]),
        Menu::new("View").items([
            MenuItem::action("Zoom In", ZoomIn).disabled(cfg.font_size >= 24.),
            MenuItem::action("Zoom Out", ZoomOut).disabled(cfg.font_size <= 9.),
            MenuItem::action("Reset Zoom", ResetZoom).checked((cfg.font_size - 12.).abs() < 0.01),
        ]),
        Menu::new("Repository").items([
            MenuItem::action("Command Palette", OpenCommandPalette),
            MenuItem::action("Undo Last Operation", OpenOperationLog),
            MenuItem::separator(),
            MenuItem::action("Bookmark Manager", OpenBookmarkManager),
            MenuItem::action("New Workspace...", NewWorkspace),
            MenuItem::separator(),
            MenuItem::action("View Remote Repository", OpenRemoteRepository),
            MenuItem::action("Show in File Manager", ShowRepoInFileManager),
            MenuItem::action(tools::open_in_editor_label(cx), OpenRepoInEditor),
            MenuItem::action(tools::open_in_terminal_label(cx), OpenRepoInTerminal),
        ]),
        Menu::new("Help").items([
            MenuItem::action("JayJay User Guide", OpenUserGuide),
            MenuItem::action("Jujutsu Documentation", OpenJujutsuDocumentation),
            MenuItem::separator(),
            MenuItem::action("Report an Issue", ReportIssue),
            MenuItem::action("Send Feedback", SendFeedback),
        ]),
    ]
}

fn app_menu() -> Menu {
    Menu::new("JayJay").items([
        MenuItem::action("About JayJay", OpenAbout),
        MenuItem::separator(),
        MenuItem::action("Settings...", OpenSettings),
        MenuItem::separator(),
        MenuItem::action("Quit JayJay", Quit),
    ])
}

fn file_menu(recent_repos: &[String]) -> Menu {
    Menu::new("File").items([
        MenuItem::action("Open Repository...", OpenRepository),
        MenuItem::submenu(open_recent_menu(recent_repos)),
    ])
}

fn open_recent_menu(recent_repos: &[String]) -> Menu {
    if recent_repos.is_empty() {
        return Menu::new("Open Recent").items([MenuItem::action(
            "No Recent Repositories",
            ClearRecentRepositories,
        )
        .disabled(true)]);
    }

    let mut items: Vec<MenuItem> = recent_repos
        .iter()
        .map(|path| {
            MenuItem::action(
                recent_repo_label(path),
                OpenRecentRepository { path: path.clone() },
            )
        })
        .collect();
    items.push(MenuItem::separator());
    items.push(MenuItem::action("Clear", ClearRecentRepositories));
    Menu::new("Open Recent").items(items)
}

fn recent_repo_label(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_owned()
}

fn register_global_actions(cx: &mut App) {
    cx.on_action(|_: &OpenSettings, cx| SettingsView::open(cx));
    cx.on_action(|_: &OpenAbout, cx| {
        SettingsView::open_section(SettingsSection::About, cx);
    });
    cx.on_action(|_: &Quit, cx| cx.quit());
    cx.on_action(|_: &OpenRepository, cx| prompt_open_repository(cx));
    cx.on_action(|action: &OpenRecentRepository, cx| {
        crate::repo::open_repo_window(PathBuf::from(&action.path), cx);
    });
    cx.on_action(|_: &ClearRecentRepositories, cx| {
        config::update(cx, |c| c.clear_recent_repos());
    });
    cx.on_action(|_: &OpenUserGuide, cx| cx.open_url(GUIDE_URL));
    cx.on_action(|_: &OpenJujutsuDocumentation, cx| cx.open_url(JUJUTSU_DOCS_URL));
    cx.on_action(|_: &ReportIssue, cx| cx.open_url(REPORT_ISSUE_URL));
    cx.on_action(|_: &SendFeedback, cx| crate::app::feedback::open(cx));
    cx.on_action(|_: &ZoomIn, cx| {
        toggle(cx, |c| {
            c.font_size = (c.font_size + 1.).min(24.);
        })
    });
    cx.on_action(|_: &ZoomOut, cx| {
        toggle(cx, |c| {
            c.font_size = (c.font_size - 1.).max(9.);
        })
    });
    cx.on_action(|_: &ResetZoom, cx| {
        toggle(cx, |c| {
            c.font_size = 12.;
        })
    });
    cx.on_action(|_: &ToggleSideBySideDiff, cx| {
        toggle(cx, |c| {
            c.diff.side_by_side ^= true;
        })
    });
    cx.on_action(|_: &ToggleIgnoreWhitespace, cx| {
        toggle(cx, |c| {
            c.diff.ignore_whitespace ^= true;
        })
    });
    cx.on_action(|_: &ToggleHideGitLfsFiles, cx| {
        toggle(cx, |c| {
            c.diff.hide_git_lfs ^= true;
        })
    });
    cx.on_action(|_: &ToggleTreeFileList, cx| {
        toggle(cx, |c| {
            c.diff.tree_file_list ^= true;
        })
    });
}

fn toggle(cx: &mut App, mutate: impl FnOnce(&mut config::AppConfig)) {
    config::update(cx, mutate);
}

pub(crate) fn prompt_open_repository(cx: &mut App) {
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: false,
        directories: true,
        multiple: false,
        prompt: Some("Choose a Jujutsu repository".into()),
    });
    cx.spawn(async move |cx| {
        let Ok(Ok(Some(paths))) = paths.await else {
            return;
        };
        let Some(path) = paths.into_iter().next() else {
            return;
        };
        cx.update(|cx| {
            crate::repo::open_repo_window(path, cx);
        });
    })
    .detach();
}

#[cfg(test)]
mod tests;
