use gpui::{
    App, AppContext, Bounds, ClipboardItem, Context, Entity, Focusable, KeyDownEvent, SharedString,
    TitlebarOptions, Window, WindowBounds, WindowKind, WindowOptions, px, size,
};

use super::actions::{ACTIONS, PaletteCtx};
use super::state::{CommandOutput, CommandPalette};
use crate::app::config::AppConfigStore;
use crate::app::theme::{Theme, observe_window_appearance};
use crate::repo::window::RepoWindow;
use crate::ui::navigation::{self, ListNav, ListNavKeys};

impl CommandPalette {
    pub fn new(
        repo_path: SharedString,
        repo_window: Option<Entity<RepoWindow>>,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.observe_global::<AppConfigStore>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            query: Default::default(),
            selected: 0,
            focus_handle: cx.focus_handle(),
            repo_path,
            repo_window,
            output: CommandOutput::Idle,
            history: Vec::new(),
            history_index: None,
            caret: Default::default(),
            focus_subscriptions: Vec::new(),
        }
    }

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
                    cx.new(|cx| Self::new(repo_path, repo_window, cx))
                },
            )
            .ok();
        if let Some(h) = handle {
            let _ = h.update(cx, |view, window, cx| {
                observe_window_appearance(window, cx);
                let f = view.focus_handle(cx);
                window.focus(&f, cx);
                view.show_caret(cx);
            });
        }
    }

    // `!` is a shorthand alias for `jj `, matching SwiftUI behavior.
    pub(super) fn parse_command(&self) -> Option<String> {
        let q = self.query.text();
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
        let q = self.query.text().trim().to_lowercase();
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
                    self.query.set_text("");
                    self.on_query_edited(cx);
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
                self.handle_line_edit_key(ev, cx);
            }
            _ => {
                if let Some(direction) =
                    navigation::list_nav_from_key(ev, ListNavKeys::COMMAND_PALETTE)
                {
                    self.handle_list_nav(direction, is_jj, visible.len(), cx);
                    return;
                }
                self.handle_line_edit_key(ev, cx);
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
        self.query.set_text(query);
        self.on_query_edited(cx);
        cx.notify();
    }

    pub fn query_text(&self) -> &str {
        self.query.text()
    }

    pub(super) fn record_command_history(&mut self, command: &str) {
        self.history = super::history::record(command, &self.history);
        self.history_index = None;
    }

    fn recall_command_history(&mut self, older: bool, cx: &mut Context<Self>) {
        let Some(recall) = super::history::recall(&self.history, self.history_index, older) else {
            return;
        };
        self.query.set_text(recall.query);
        self.history_index = recall.index;
        self.selected = 0;
        self.output = CommandOutput::Idle;
        self.show_caret(cx);
        cx.notify();
    }

    fn on_query_edited(&mut self, cx: &mut Context<Self>) {
        self.selected = 0;
        self.history_index = None;
        if !matches!(self.output, CommandOutput::Running { .. }) {
            self.output = CommandOutput::Idle;
        }
        self.show_caret(cx);
    }

    fn handle_line_edit_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let clipboard_text = cx.read_from_clipboard().and_then(|item| item.text());
        let result = self.query.handle_key(ev, clipboard_text.as_deref());
        if !result.handled {
            return;
        }
        if let Some(text) = result.copy_to_clipboard {
            cx.write_to_clipboard(ClipboardItem::new_string(text));
        }
        if result.changed {
            self.on_query_edited(cx);
        } else {
            self.show_caret(cx);
        }
    }

    pub(super) fn ensure_focus_handlers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_subscriptions.is_empty() {
            return;
        }
        let focus_handle = self.focus_handle.clone();
        self.focus_subscriptions = vec![
            cx.on_focus(&focus_handle, window, |palette, _window, cx| {
                palette.show_caret(cx);
            }),
            cx.on_blur(&focus_handle, window, |palette, _window, cx| {
                palette.hide_caret(cx);
            }),
        ];
        if self.focus_handle.is_focused(window) {
            self.show_caret(cx);
        }
    }

    pub(super) fn caret_visible(&self) -> bool {
        self.caret.visible()
    }

    fn show_caret(&mut self, cx: &mut Context<Self>) {
        self.caret.show(cx, |palette, generation, cx| {
            palette.toggle_caret(generation, cx)
        });
    }

    fn hide_caret(&mut self, cx: &mut Context<Self>) {
        self.caret.hide(cx);
    }

    fn toggle_caret(&mut self, generation: u64, cx: &mut Context<Self>) -> bool {
        self.caret.toggle_if_current(generation, cx)
    }
}
