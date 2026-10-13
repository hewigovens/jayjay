use super::{ContextAction, ContextMenuState};
use crate::repo::window::{ChangeAction, FileBatchAction};

impl ContextAction {
    pub(super) fn shortcut(&self) -> Option<char> {
        match self {
            Self::NewChangeOnTop(_) => Some('n'),
            Self::CreateBookmark(_) | Self::ToggleAnnotateFor(_) => Some('b'),
            Self::CreateTag(_) => Some('t'),
            Self::EditDescription { .. } => Some('d'),
            Self::OpenEvologFor(_) => Some('v'),
            Self::AbandonChange(_) => Some('a'),
            Self::Change(action) => match action.as_ref() {
                ChangeAction::Edit { .. } => Some('e'),
                ChangeAction::Squash { into: None, .. } => Some('s'),
                ChangeAction::RebaseOntoTrunk { .. } => Some('r'),
                ChangeAction::AbandonMany { .. } => Some('a'),
                _ => None,
            },
            Self::OpenInEditor { .. } => Some('o'),
            Self::ShowInFileManager(_) => Some('f'),
            Self::OpenFileHistoryFor(_) => Some('h'),
            Self::FileBatch(action) => match action.as_ref() {
                FileBatchAction::SetReviewed { .. } => Some('r'),
                FileBatchAction::Split(_) => Some('s'),
                _ => None,
            },
            _ => None,
        }
    }
}

impl ContextMenuState {
    pub(crate) fn handle_key(&mut self, key: &str) -> Option<ContextAction> {
        if matches!(key, "left" | "escape") && self.submenu_index.is_some() {
            self.submenu_index = None;
            self.submenu_selected_index = None;
            return None;
        }
        let in_submenu = self.submenu_index.is_some();
        let (items, selected) = if let Some(index) = self.submenu_index {
            (
                self.items[index].submenu_items()?,
                &mut self.submenu_selected_index,
            )
        } else {
            (self.items.as_slice(), &mut self.selected_index)
        };
        let eligible: Vec<_> = items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.enabled && !item.is_separator)
            .map(|(index, _)| index)
            .collect();
        if eligible.is_empty() {
            return None;
        }
        match key {
            "up" | "down" | "home" | "end" => {
                let current = selected.and_then(|index| eligible.iter().position(|i| *i == index));
                let next = match key {
                    "home" => 0,
                    "end" => eligible.len() - 1,
                    "up" => current.map_or(eligible.len() - 1, |i| {
                        (i + eligible.len() - 1) % eligible.len()
                    }),
                    _ => current.map_or(0, |i| (i + 1) % eligible.len()),
                };
                *selected = Some(eligible[next]);
                return None;
            }
            "enter" | "space" | "right" => {}
            _ => {
                let mut chars = key.chars();
                let shortcut = chars.next()?;
                if chars.next().is_some() {
                    return None;
                }
                *selected = Some(
                    *eligible
                        .iter()
                        .find(|index| items[**index].shortcut == Some(shortcut))?,
                );
            }
        }
        let index = (*selected)?;
        let item = &items[index];
        if !item.enabled || item.is_separator {
            return None;
        }
        if let Some(children) = item.submenu_items() {
            if !in_submenu {
                self.submenu_index = Some(index);
                self.submenu_selected_index = children
                    .iter()
                    .position(|item| item.enabled && !item.is_separator);
            }
            None
        } else if key == "right" {
            None
        } else {
            Some(item.action.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::context_menu::ContextMenuItem;

    #[test]
    fn menu_navigation_skips_disabled_actions_and_scopes_letters_to_the_open_panel() {
        let copy = |text: &'static str| {
            ContextMenuItem::new(text, "", ContextAction::CopyText(text.into())).with_shortcut('c')
        };
        let mut menu = ContextMenuState {
            anchor: Default::default(),
            items: vec![
                copy("disabled").with_enabled(false),
                ContextMenuItem::separator(),
                copy("main"),
                ContextMenuItem::submenu(
                    "More",
                    "",
                    vec![copy("disabled child").with_enabled(false), copy("child")],
                ),
            ],
            submenu_index: None,
            selected_index: None,
            submenu_selected_index: None,
        };
        menu.handle_key("home");
        assert_eq!(menu.selected_index, Some(2));
        menu.handle_key("up");
        assert_eq!(menu.selected_index, Some(3));
        menu.handle_key("right");
        assert_eq!(menu.submenu_selected_index, Some(1));
        assert!(
            matches!(menu.handle_key("c"), Some(ContextAction::CopyText(text)) if text == "child")
        );
        menu.handle_key("escape");
        assert_eq!(menu.submenu_index, None);
        assert_eq!(menu.selected_index, Some(3));
        assert!(
            matches!(menu.handle_key("c"), Some(ContextAction::CopyText(text)) if text == "main")
        );
        menu.items[2].enabled = false;
        assert!(menu.handle_key("c").is_none());
        assert!(menu.handle_key("enter").is_none());
    }
}
