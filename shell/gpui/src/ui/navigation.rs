use gpui::KeyDownEvent;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListNav {
    Previous,
    Next,
}

impl ListNav {
    fn delta(self) -> i32 {
        match self {
            Self::Previous => -1,
            Self::Next => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ListNavKeys {
    arrows: bool,
    vim: bool,
    ctrl_np: bool,
}

impl ListNavKeys {
    pub(crate) const COMMAND_PALETTE: Self = Self {
        arrows: true,
        vim: false,
        ctrl_np: true,
    };

    pub(crate) const CONTENT_LIST: Self = Self {
        arrows: true,
        vim: true,
        ctrl_np: true,
    };
}

pub(crate) fn list_nav_from_key(ev: &KeyDownEvent, keys: ListNavKeys) -> Option<ListNav> {
    let modifiers = &ev.keystroke.modifiers;
    if modifiers.platform || modifiers.alt {
        return None;
    }

    let key = ev.keystroke.key.as_str();
    if modifiers.control {
        if !keys.ctrl_np {
            return None;
        }
        return match key {
            "p" => Some(ListNav::Previous),
            "n" => Some(ListNav::Next),
            _ => None,
        };
    }

    if keys.arrows {
        match key {
            "up" => return Some(ListNav::Previous),
            "down" => return Some(ListNav::Next),
            _ => {}
        }
    }

    if keys.vim {
        match key {
            "k" => return Some(ListNav::Previous),
            "j" => return Some(ListNav::Next),
            _ => {}
        }
    }

    None
}

pub(crate) fn move_index(current: Option<usize>, len: usize, direction: ListNav) -> Option<usize> {
    if len == 0 {
        return None;
    }
    let current = current.unwrap_or(0).min(len - 1) as i32;
    Some((current + direction.delta()).clamp(0, len as i32 - 1) as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn move_index_clamps_to_list_bounds() {
        assert_eq!(move_index(Some(0), 3, ListNav::Previous), Some(0));
        assert_eq!(move_index(Some(0), 3, ListNav::Next), Some(1));
        assert_eq!(move_index(Some(2), 3, ListNav::Next), Some(2));
        assert_eq!(move_index(None, 3, ListNav::Next), Some(1));
        assert_eq!(move_index(Some(0), 0, ListNav::Next), None);
    }
}
