use gpui::{KeyBinding, actions};

actions!(
    text_area,
    [
        Backspace,
        Delete,
        DeleteToLineStart,
        DeleteToLineEnd,
        DeletePreviousWord,
        Left,
        Right,
        Up,
        Down,
        WordLeft,
        WordRight,
        SelectLeft,
        SelectRight,
        SelectUp,
        SelectDown,
        SelectWordLeft,
        SelectWordRight,
        SelectHome,
        SelectEnd,
        DocumentStart,
        DocumentEnd,
        SelectDocumentStart,
        SelectDocumentEnd,
        SelectAll,
        Home,
        End,
        Newline,
        Paste,
        Cut,
        Copy
    ]
);

pub fn key_bindings(mod_key: &str) -> Vec<KeyBinding> {
    let mut bindings = vec![
        KeyBinding::new("backspace", Backspace, Some("TextArea")),
        KeyBinding::new("delete", Delete, Some("TextArea")),
        KeyBinding::new("left", Left, Some("TextArea")),
        KeyBinding::new("right", Right, Some("TextArea")),
        KeyBinding::new("up", Up, Some("TextArea")),
        KeyBinding::new("down", Down, Some("TextArea")),
        KeyBinding::new("alt-left", WordLeft, Some("TextArea")),
        KeyBinding::new("alt-right", WordRight, Some("TextArea")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextArea")),
        KeyBinding::new("shift-right", SelectRight, Some("TextArea")),
        KeyBinding::new("shift-up", SelectUp, Some("TextArea")),
        KeyBinding::new("shift-down", SelectDown, Some("TextArea")),
        KeyBinding::new("alt-shift-left", SelectWordLeft, Some("TextArea")),
        KeyBinding::new("alt-shift-right", SelectWordRight, Some("TextArea")),
        KeyBinding::new("alt-backspace", DeletePreviousWord, Some("TextArea")),
        KeyBinding::new("alt-delete", DeletePreviousWord, Some("TextArea")),
        KeyBinding::new("home", Home, Some("TextArea")),
        KeyBinding::new("end", End, Some("TextArea")),
        KeyBinding::new("shift-home", SelectHome, Some("TextArea")),
        KeyBinding::new("shift-end", SelectEnd, Some("TextArea")),
        KeyBinding::new("enter", Newline, Some("TextArea")),
        KeyBinding::new("shift-enter", Newline, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-a").as_str(), SelectAll, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-v").as_str(), Paste, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-x").as_str(), Cut, Some("TextArea")),
        KeyBinding::new(format!("{mod_key}-c").as_str(), Copy, Some("TextArea")),
    ];
    if mod_key == "cmd" {
        bindings.extend([
            KeyBinding::new("ctrl-a", Home, Some("TextArea")),
            KeyBinding::new("ctrl-e", End, Some("TextArea")),
            KeyBinding::new("ctrl-p", Up, Some("TextArea")),
            KeyBinding::new("ctrl-n", Down, Some("TextArea")),
            KeyBinding::new("ctrl-u", DeleteToLineStart, Some("TextArea")),
            KeyBinding::new("ctrl-k", DeleteToLineEnd, Some("TextArea")),
            KeyBinding::new("cmd-left", Home, Some("TextArea")),
            KeyBinding::new("cmd-right", End, Some("TextArea")),
            KeyBinding::new("cmd-up", DocumentStart, Some("TextArea")),
            KeyBinding::new("cmd-down", DocumentEnd, Some("TextArea")),
            KeyBinding::new("cmd-shift-left", SelectHome, Some("TextArea")),
            KeyBinding::new("cmd-shift-right", SelectEnd, Some("TextArea")),
            KeyBinding::new("cmd-shift-up", SelectDocumentStart, Some("TextArea")),
            KeyBinding::new("cmd-shift-down", SelectDocumentEnd, Some("TextArea")),
            KeyBinding::new("cmd-backspace", DeleteToLineStart, Some("TextArea")),
            KeyBinding::new("cmd-delete", DeleteToLineStart, Some("TextArea")),
        ]);
    }
    bindings
}
