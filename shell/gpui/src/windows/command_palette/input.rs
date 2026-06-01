use gpui::{
    App, AppContext, Bounds, Context, Entity, Focusable, KeyDownEvent, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowKind, WindowOptions, px, size,
};

use super::actions::{ACTIONS, PaletteCtx};
use super::state::{CommandOutput, CommandPalette};
use crate::app::config::AppConfigStore;
use crate::app::theme::Theme;
use crate::repo::window::RepoWindow;
use crate::ui::navigation::{self, ListNav, ListNavKeys};

impl CommandPalette {
    pub fn open(repo_path: SharedString, repo_window: Option<Entity<RepoWindow>>, cx: &mut App) {
        let bounds = Bounds::centered(None, size(px(640.), px(480.)), cx);
        let handle = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(TitlebarOptions {
                        title: Some("JayJay Command Palette".into()),
                        ..Default::default()
                    }),
                    kind: WindowKind::PopUp,
                    ..Default::default()
                },
                |_, cx| {
                    let repo_path = repo_path.clone();
                    let repo_window = repo_window.clone();
                    cx.new(|cx| {
                        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
                            .detach();
                        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
                        Self {
                            query: String::new(),
                            selected: 0,
                            focus_handle: cx.focus_handle(),
                            repo_path,
                            repo_window,
                            output: CommandOutput::Idle,
                            history: Vec::new(),
                            history_index: None,
                        }
                    })
                },
            )
            .ok();
        if let Some(h) = handle {
            let _ = h.update(cx, |view, window, cx| {
                crate::app::theme::observe_window_appearance(window, cx);
                let f = view.focus_handle(cx);
                window.focus(&f, cx);
            });
        }
    }

    // `!` is a shorthand alias for `jj `, matching SwiftUI behavior.
    pub(super) fn parse_command(&self) -> Option<String> {
        let q = self.query.as_str();
        let body_after = |rest: &str| rest.trim_start().to_string();
        if q == "jj" || q == "!" {
            return Some(String::new());
        }
        if let Some(rest) = q.strip_prefix("jj ") {
            return Some(body_after(rest));
        }
        if let Some(rest) = q.strip_prefix('!') {
            return Some(body_after(rest));
        }
        None
    }

    pub(super) fn matches(&self) -> Vec<usize> {
        let q = self.query.trim().to_lowercase();
        if q.is_empty() {
            return (0..ACTIONS.len()).collect();
        }
        ACTIONS
            .iter()
            .enumerate()
            .filter(|(_, a)| {
                a.name.to_lowercase().contains(&q)
                    || a.keywords.iter().any(|k| k.to_lowercase().contains(&q))
            })
            .map(|(i, _)| i)
            .collect()
    }

    pub(super) fn on_key(
        &mut self,
        ev: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let key = ev.keystroke.key.as_str();
        let visible = self.matches();
        let is_jj = self.parse_command().is_some();

        match key {
            "escape" => {
                if matches!(self.output, CommandOutput::Done { .. }) {
                    self.output = CommandOutput::Idle;
                    self.query.clear();
                } else {
                    window.remove_window();
                    return;
                }
            }
            "enter" => {
                if let Some(body) = self.parse_command() {
                    self.run_command(body, cx);
                    return;
                }
                if let Some(&action_ix) = visible.get(self.selected) {
                    self.dispatch_action(action_ix, window, cx);
                    return;
                }
            }
            "backspace" => {
                self.query.pop();
                self.on_query_edited();
            }
            _ => {
                if let Some(direction) =
                    navigation::list_nav_from_key(ev, ListNavKeys::COMMAND_PALETTE)
                {
                    self.handle_list_nav(direction, is_jj, visible.len(), cx);
                    return;
                }
                if let Some(c) = ev.keystroke.key_char.as_ref() {
                    let m = &ev.keystroke.modifiers;
                    if !m.platform && !m.control && !m.alt {
                        self.query.push_str(c);
                        self.on_query_edited();
                    }
                }
            }
        }
        cx.notify();
    }

    fn handle_list_nav(
        &mut self,
        direction: ListNav,
        is_jj: bool,
        visible_len: usize,
        cx: &mut Context<Self>,
    ) {
        if is_jj {
            self.recall_command_history(matches!(direction, ListNav::Previous), cx);
            return;
        }
        if let Some(next) = navigation::move_index(Some(self.selected), visible_len, direction) {
            self.selected = next;
            cx.notify();
        }
    }

    pub(super) fn dispatch_action(
        &mut self,
        action_ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let dispatch = ACTIONS[action_ix].dispatch;
        let ctx = PaletteCtx {
            repo_path: self.repo_path.as_ref(),
            repo_window: self.repo_window.clone(),
        };
        window.remove_window();
        dispatch(&ctx, cx);
    }

    pub(super) fn set_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.query = query;
        self.on_query_edited();
        cx.notify();
    }

    pub(super) fn record_command_history(&mut self, command: &str) {
        self.history = super::history::record(command, &self.history);
        self.history_index = None;
    }

    fn recall_command_history(&mut self, older: bool, cx: &mut Context<Self>) {
        let Some(recall) = super::history::recall(&self.history, self.history_index, older) else {
            return;
        };
        self.query = recall.query;
        self.history_index = recall.index;
        self.selected = 0;
        self.output = CommandOutput::Idle;
        cx.notify();
    }

    fn on_query_edited(&mut self) {
        self.selected = 0;
        self.history_index = None;
        if !matches!(self.output, CommandOutput::Running { .. }) {
            self.output = CommandOutput::Idle;
        }
    }
}
