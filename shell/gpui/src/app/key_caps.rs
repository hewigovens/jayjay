use gpui::{Action, App, Modifiers};

#[derive(Debug)]
pub struct KeyCaps(Vec<String>);

impl KeyCaps {
    pub fn for_action(action: &dyn Action, cx: &App) -> Option<Self> {
        let keymap = cx.key_bindings();
        let keymap = keymap.borrow();
        let binding = keymap.bindings_for_action(action).last()?;
        let [keystroke] = binding.keystrokes() else {
            return None;
        };
        let mut caps = modifier_caps(keystroke.modifiers());
        caps.push(key_cap(keystroke.key()));
        Some(Self(caps))
    }

    pub fn label(action: &dyn Action, cx: &App) -> String {
        Self::for_action(action, cx)
            .map(|caps| caps.to_string())
            .unwrap_or_default()
    }

    pub fn caps(&self) -> &[String] {
        &self.0
    }
}

impl std::fmt::Display for KeyCaps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let separator = if cfg!(target_os = "macos") { "" } else { "+" };
        f.write_str(&self.0.join(separator))
    }
}

fn modifier_caps(modifiers: &Modifiers) -> Vec<String> {
    let labels: [(bool, &str, &str); 4] = [
        (modifiers.control, "⌃", "Ctrl"),
        (modifiers.alt, "⌥", "Alt"),
        (modifiers.shift, "⇧", "Shift"),
        (modifiers.platform, "⌘", "Super"),
    ];
    labels
        .into_iter()
        .filter(|(held, _, _)| *held)
        .map(|(_, mac, other)| {
            if cfg!(target_os = "macos") {
                mac
            } else {
                other
            }
            .to_owned()
        })
        .collect()
}

fn key_cap(key: &str) -> String {
    let mac = cfg!(target_os = "macos");
    match key {
        "enter" if mac => "↩".to_owned(),
        "enter" => "Enter".to_owned(),
        "escape" => "Esc".to_owned(),
        "space" => "Space".to_owned(),
        "left" => "←".to_owned(),
        "right" => "→".to_owned(),
        "up" => "↑".to_owned(),
        "down" => "↓".to_owned(),
        "-" => "−".to_owned(),
        other => other.to_uppercase(),
    }
}
