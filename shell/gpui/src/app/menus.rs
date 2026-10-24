use std::path::{Path, PathBuf};

use gpui::{App, Menu, MenuItem, PathPromptOptions};

use super::actions::{
    ClearRecentRepositories, CloneRepository, FilterByRevset, NewWorkspace,
    NewWorkspaceFromPullRequest, OpenAbout, OpenBookmarkManager, OpenCommandPalette, OpenFind,
    OpenJujutsuDocumentation, OpenKeyboardShortcuts, OpenOperationLog, OpenOverview,
    OpenRecentRepository, OpenRemoteRepository, OpenRepoInEditor, OpenRepoInTerminal,
    OpenRepository, OpenSettings, OpenUserGuide, Quit, ReportIssue, ResetZoom, RevealLogs,
    SendFeedback, ShowRepoInFileManager, ToggleHideGitLfsFiles, ToggleIgnoreWhitespace,
    ToggleSideBySideDiff, ToggleSidebar, ToggleTreeFileList, ZoomIn, ZoomOut,
};
use super::config::{self, AppConfig, current};
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
        Menu::new(jayjay_i18n::menu::edit()).items([MenuItem::action(
            jayjay_i18n::menu::find_ellipsis(),
            OpenFind,
        )]),
        Menu::new(jayjay_i18n::menu::view()).items([
            MenuItem::action(
                if cfg.layout.sidebar_hidden {
                    jayjay_i18n::menu::show_sidebar()
                } else {
                    jayjay_i18n::menu::hide_sidebar()
                },
                ToggleSidebar,
            ),
            MenuItem::separator(),
            MenuItem::action(jayjay_i18n::menu::zoom_in(), ZoomIn).disabled(
                cfg.interface_font_size().min(cfg.code_font_size()) >= AppConfig::MAX_FONT_SIZE,
            ),
            MenuItem::action(jayjay_i18n::menu::zoom_out(), ZoomOut).disabled(
                cfg.interface_font_size().max(cfg.code_font_size()) <= AppConfig::MIN_FONT_SIZE,
            ),
            MenuItem::action(jayjay_i18n::menu::reset_zoom(), ResetZoom)
                .checked(cfg.is_default_zoom()),
        ]),
        Menu::new(jayjay_i18n::menu::repository()).items([
            MenuItem::action(jayjay_i18n::menu::command_palette(), OpenCommandPalette),
            MenuItem::action(jayjay_i18n::menu::undo_last_operation(), OpenOperationLog),
            MenuItem::separator(),
            MenuItem::action(
                jayjay_i18n::menu::filter_by_revset_ellipsis(),
                FilterByRevset,
            ),
            MenuItem::action(jayjay_i18n::menu::bookmark_manager(), OpenBookmarkManager),
            MenuItem::action(jayjay_i18n::menu::repo_overview(), OpenOverview),
            MenuItem::action(jayjay_i18n::menu::new_workspace_ellipsis(), NewWorkspace),
            MenuItem::action(
                jayjay_i18n::menu::new_workspace_from_pull_request_ellipsis(),
                NewWorkspaceFromPullRequest,
            ),
            MenuItem::separator(),
            MenuItem::action(
                jayjay_i18n::menu::view_remote_repository(),
                OpenRemoteRepository,
            ),
            MenuItem::action(
                crate::platform::show_in_file_manager_label(),
                ShowRepoInFileManager,
            ),
            MenuItem::action(tools::open_in_editor_label(cx), OpenRepoInEditor),
            MenuItem::action(tools::open_in_terminal_label(cx), OpenRepoInTerminal),
        ]),
        Menu::new(jayjay_i18n::menu::help()).items([
            MenuItem::action(
                jayjay_i18n::menu::keyboard_shortcuts(),
                OpenKeyboardShortcuts,
            ),
            MenuItem::separator(),
            MenuItem::action(jayjay_i18n::menu::jayjay_user_guide(), OpenUserGuide),
            MenuItem::action(
                jayjay_i18n::menu::jujutsu_documentation(),
                OpenJujutsuDocumentation,
            ),
            MenuItem::separator(),
            MenuItem::action(jayjay_i18n::menu::report_an_issue(), ReportIssue),
            MenuItem::action(jayjay_i18n::menu::send_feedback(), SendFeedback),
            MenuItem::action(jayjay_i18n::menu::reveal_logs(), RevealLogs),
        ]),
    ]
}

fn app_menu() -> Menu {
    Menu::new("JayJay").items([
        MenuItem::action(jayjay_i18n::menu::about_jayjay(), OpenAbout),
        MenuItem::separator(),
        MenuItem::action(jayjay_i18n::menu::settings_ellipsis(), OpenSettings),
        MenuItem::separator(),
        MenuItem::action(jayjay_i18n::menu::quit_jayjay(), Quit),
    ])
}

fn file_menu(recent_repos: &[String]) -> Menu {
    Menu::new(jayjay_i18n::menu::file()).items([
        MenuItem::action(
            jayjay_i18n::menu::open_repository_ellipsis(),
            OpenRepository,
        ),
        MenuItem::action(
            jayjay_i18n::menu::clone_repository_ellipsis(),
            CloneRepository,
        ),
        MenuItem::submenu(open_recent_menu(recent_repos)),
    ])
}

fn open_recent_menu(recent_repos: &[String]) -> Menu {
    if recent_repos.is_empty() {
        return Menu::new(jayjay_i18n::menu::open_recent()).items([MenuItem::action(
            jayjay_i18n::menu::no_recent_repositories(),
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
    items.push(MenuItem::action(
        jayjay_i18n::common::clear(),
        ClearRecentRepositories,
    ));
    Menu::new(jayjay_i18n::menu::open_recent()).items(items)
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
    cx.on_action(|_: &CloneRepository, cx| {
        crate::windows::clone_repository::CloneRepositoryView::open(cx);
    });
    cx.on_action(|action: &OpenRecentRepository, cx| {
        crate::repo::open_repo_window(PathBuf::from(&action.path), cx);
    });
    cx.on_action(|_: &ClearRecentRepositories, cx| {
        config::update(cx, |c| c.clear_recent_repos());
    });
    cx.on_action(|_: &OpenUserGuide, cx| crate::app::links::open_url(cx, GUIDE_URL));
    cx.on_action(|_: &OpenKeyboardShortcuts, cx| {
        crate::windows::keyboard_shortcuts::KeyboardShortcutsView::open(cx);
    });
    cx.on_action(|_: &OpenJujutsuDocumentation, cx| {
        crate::app::links::open_url(cx, JUJUTSU_DOCS_URL)
    });
    cx.on_action(|_: &ReportIssue, cx| crate::app::links::open_url(cx, REPORT_ISSUE_URL));
    cx.on_action(|_: &SendFeedback, cx| crate::app::feedback::open(cx));
    cx.on_action(|_: &RevealLogs, _| {
        if let Some(path) = jayjay_core::LogFile::path() {
            crate::platform::reveal_path(&path);
        }
    });
    cx.on_action(|_: &ZoomIn, cx| toggle(cx, |c| c.zoom(1.)));
    cx.on_action(|_: &ZoomOut, cx| toggle(cx, |c| c.zoom(-1.)));
    cx.on_action(|_: &ResetZoom, cx| toggle(cx, AppConfig::reset_zoom));
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
        prompt: Some(jayjay_i18n::menu::choose_a_jujutsu_repository().into()),
    });
    cx.spawn(async move |cx| {
        let paths = match paths.await {
            Ok(Ok(Some(paths))) => paths,
            Ok(Ok(None)) | Err(_) => return,
            Ok(Err(error)) => {
                log::error!("path picker unavailable: {error}");
                cx.update(crate::windows::open_repository::OpenRepositoryPathView::open);
                return;
            }
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
