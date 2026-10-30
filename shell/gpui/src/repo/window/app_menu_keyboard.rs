use gpui::{Context, Keystroke, Window};

use super::RepoWindow;
#[cfg(not(target_os = "macos"))]
use crate::ui::app_menu::MenuRow;

impl RepoWindow {
    /// Arrow keys walk the open menu bar menu, Enter runs the selected item; Escape is the window's Dismiss.
    #[cfg(not(target_os = "macos"))]
    pub(super) fn handle_app_menu_key(
        &mut self,
        key: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(menu) = self.app_menu.as_ref() else {
            return false;
        };
        // A shortcut moves on to whatever it opens, so the menu shouldn't keep claiming keys.
        if key.modifiers.modified() {
            self.close_app_menu(cx);
            return false;
        }
        let rows = MenuRow::for_menu(&menu.menu_name, cx);
        match key.key.as_str() {
            "down" | "up" => {
                let selected = MenuRow::step(&rows, menu.selected, key.key == "down");
                if let Some(menu) = self.app_menu.as_mut() {
                    menu.selected = selected;
                }
                cx.notify();
            }
            "enter" => {
                let Some(MenuRow::Action { action, .. }) = menu
                    .selected
                    .and_then(|ix| rows.get(ix))
                    .filter(|row| row.is_selectable())
                else {
                    return false;
                };
                let action = action.boxed_clone();
                self.close_app_menu(cx);
                window.dispatch_action(action, cx);
            }
            "left" | "right" => {
                let names: Vec<_> = cx
                    .get_menus()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|menu| menu.name)
                    .collect();
                let Some(current) = names.iter().position(|name| *name == menu.menu_name) else {
                    return true;
                };
                let next = if key.key == "right" {
                    (current + 1) % names.len()
                } else {
                    current.checked_sub(1).unwrap_or(names.len() - 1)
                };
                let name = names[next].clone();
                let first = MenuRow::step(&MenuRow::for_menu(&name, cx), None, true);
                self.open_named_app_menu(name, first, cx);
            }
            _ => return false,
        }
        true
    }

    #[cfg(target_os = "macos")]
    pub(super) fn handle_app_menu_key(
        &mut self,
        _key: &Keystroke,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> bool {
        false
    }

    #[cfg(not(target_os = "macos"))]
    pub(crate) fn select_app_menu_row(&mut self, ix: usize, cx: &mut Context<Self>) {
        if let Some(menu) = self.app_menu.as_mut()
            && menu.selected != Some(ix)
        {
            menu.selected = Some(ix);
            cx.notify();
        }
    }
}
