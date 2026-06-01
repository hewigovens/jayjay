use gpui::KeyDownEvent;

use super::LineEdit;
use crate::ui::input::{
    next_boundary, next_word_boundary, previous_boundary, previous_word_boundary,
    sanitize_single_line,
};

#[derive(Debug, Default)]
pub struct LineEditKeyResult {
    pub handled: bool,
    pub changed: bool,
    pub copy_to_clipboard: Option<String>,
}

impl LineEditKeyResult {
    fn handled(changed: bool) -> Self {
        Self {
            handled: true,
            changed,
            copy_to_clipboard: None,
        }
    }

    fn copied(text: String, changed: bool) -> Self {
        Self {
            handled: true,
            changed,
            copy_to_clipboard: Some(text),
        }
    }
}

impl LineEdit {
    pub fn handle_key(
        &mut self,
        ev: &KeyDownEvent,
        clipboard_text: Option<&str>,
    ) -> LineEditKeyResult {
        let key = ev.keystroke.key.as_str();
        let m = &ev.keystroke.modifiers;

        if m.platform && !m.alt && !m.control {
            match key {
                "a" => {
                    self.select_all();
                    return LineEditKeyResult::handled(false);
                }
                "v" => {
                    if let Some(text) = clipboard_text {
                        self.replace_selection(&sanitize_single_line(text));
                        return LineEditKeyResult::handled(true);
                    }
                    return LineEditKeyResult::handled(false);
                }
                "c" => {
                    return self
                        .selected_text()
                        .map(|text| LineEditKeyResult::copied(text, false))
                        .unwrap_or_else(|| LineEditKeyResult::handled(false));
                }
                "x" => {
                    return self
                        .selected_text()
                        .map(|text| {
                            self.replace_selection("");
                            LineEditKeyResult::copied(text, true)
                        })
                        .unwrap_or_else(|| LineEditKeyResult::handled(false));
                }
                "backspace" | "delete" => {
                    return LineEditKeyResult::handled(self.delete_to_start());
                }
                _ => {}
            }
        }

        if m.control && !m.platform && !m.alt {
            match key {
                "a" => {
                    self.move_to(0);
                    return LineEditKeyResult::handled(false);
                }
                "e" => {
                    self.move_to(self.text.len());
                    return LineEditKeyResult::handled(false);
                }
                _ => {}
            }
        }

        if m.alt && m.shift && !m.platform && !m.control {
            match key {
                "left" => {
                    self.select_to(previous_word_boundary(&self.text, self.cursor_offset()));
                    return LineEditKeyResult::handled(false);
                }
                "right" => {
                    self.select_to(next_word_boundary(&self.text, self.cursor_offset()));
                    return LineEditKeyResult::handled(false);
                }
                _ => {}
            }
        }

        if m.alt && !m.platform && !m.control {
            match key {
                "left" => {
                    self.move_word_left();
                    return LineEditKeyResult::handled(false);
                }
                "right" => {
                    self.move_word_right();
                    return LineEditKeyResult::handled(false);
                }
                "backspace" | "delete" => {
                    return LineEditKeyResult::handled(self.delete_previous_word());
                }
                _ => {}
            }
        }

        if m.shift && !m.platform && !m.control && !m.alt {
            match key {
                "left" => {
                    self.select_to(previous_boundary(&self.text, self.cursor_offset()));
                    return LineEditKeyResult::handled(false);
                }
                "right" => {
                    self.select_to(next_boundary(&self.text, self.cursor_offset()));
                    return LineEditKeyResult::handled(false);
                }
                _ => {}
            }
        }

        if m.platform || m.control || m.alt {
            return LineEditKeyResult::default();
        }

        match key {
            "left" => {
                self.move_left();
                LineEditKeyResult::handled(false)
            }
            "right" => {
                self.move_right();
                LineEditKeyResult::handled(false)
            }
            "home" => {
                self.move_to(0);
                LineEditKeyResult::handled(false)
            }
            "end" => {
                self.move_to(self.text.len());
                LineEditKeyResult::handled(false)
            }
            "backspace" => LineEditKeyResult::handled(self.backspace()),
            "delete" => LineEditKeyResult::handled(self.delete()),
            _ => {
                if let Some(c) = ev.keystroke.key_char.as_ref()
                    && !c.is_empty()
                    && c.chars().all(|ch| !ch.is_control())
                {
                    self.replace_selection(c);
                    return LineEditKeyResult::handled(true);
                }
                LineEditKeyResult::default()
            }
        }
    }
}
