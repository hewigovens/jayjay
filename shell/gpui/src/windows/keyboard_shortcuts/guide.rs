pub(super) struct ShortcutEntry {
    pub(super) label: &'static str,
    pub(super) keys: &'static [&'static str],
}

pub(super) struct ShortcutSection {
    pub(super) title: &'static str,
    pub(super) entries: &'static [ShortcutEntry],
}

pub(super) const SECTIONS: &[ShortcutSection] = &[
    ShortcutSection {
        title: "General",
        entries: &[
            ShortcutEntry {
                label: "Open Repository",
                keys: &["Mod", "O"],
            },
            ShortcutEntry {
                label: "Command Palette",
                keys: &["Shift", "Mod", "P"],
            },
            ShortcutEntry {
                label: "Refresh",
                keys: &["Mod", "R"],
            },
            ShortcutEntry {
                label: "Keyboard Shortcuts",
                keys: &["Mod", "/"],
            },
            ShortcutEntry {
                label: "Settings",
                keys: &["Mod", ","],
            },
            ShortcutEntry {
                label: "Close Window",
                keys: &["Mod", "W"],
            },
        ],
    },
    ShortcutSection {
        title: "View",
        entries: &[
            ShortcutEntry {
                label: "Zoom In",
                keys: &["Mod", "+"],
            },
            ShortcutEntry {
                label: "Zoom Out",
                keys: &["Mod", "−"],
            },
            ShortcutEntry {
                label: "Reset Zoom",
                keys: &["Mod", "0"],
            },
        ],
    },
    ShortcutSection {
        title: "Navigation",
        entries: &[
            ShortcutEntry {
                label: "Next / Previous Item",
                keys: &["J", "K"],
            },
            ShortcutEntry {
                label: "Move Up / Down",
                keys: &["↑", "↓"],
            },
            ShortcutEntry {
                label: "Alternate Up / Down",
                keys: &["Ctrl", "P", "/", "N"],
            },
            ShortcutEntry {
                label: "Switch Pane",
                keys: &["Tab"],
            },
        ],
    },
    ShortcutSection {
        title: "Repository",
        entries: &[
            ShortcutEntry {
                label: "Bookmark Manager",
                keys: &["Shift", "Mod", "B"],
            },
            ShortcutEntry {
                label: "Undo Last Operation",
                keys: &["Shift", "Mod", "U"],
            },
            ShortcutEntry {
                label: "Show in File Manager",
                keys: &["Alt", "Mod", "F"],
            },
        ],
    },
    ShortcutSection {
        title: "Diff & Review",
        entries: &[
            ShortcutEntry {
                label: "Find in Diff",
                keys: &["Mod", "F"],
            },
            ShortcutEntry {
                label: "Copy Diff Selection",
                keys: &["Mod", "C"],
            },
            ShortcutEntry {
                label: "Mark File Reviewed",
                keys: &["Space"],
            },
            ShortcutEntry {
                label: "Save Review Note",
                keys: &["Mod", "Return"],
            },
            ShortcutEntry {
                label: "Save Edited File",
                keys: &["Mod", "S"],
            },
            ShortcutEntry {
                label: "Expand All Files",
                keys: &["Alt", "Mod", "E"],
            },
            ShortcutEntry {
                label: "Collapse All Files",
                keys: &["Alt", "Mod", "C"],
            },
            ShortcutEntry {
                label: "Collapse / Expand File",
                keys: &["←", "→"],
            },
            ShortcutEntry {
                label: "Toggle File",
                keys: &["Return"],
            },
        ],
    },
];

pub(super) fn columns() -> [&'static [ShortcutSection]; 2] {
    let target = SECTIONS
        .iter()
        .map(|section| section.entries.len())
        .sum::<usize>()
        / 2;
    let mut left_entries = 0;
    let split = SECTIONS
        .iter()
        .position(|section| {
            if left_entries >= target {
                true
            } else {
                left_entries += section.entries.len();
                false
            }
        })
        .unwrap_or(SECTIONS.len());
    let (left, right) = SECTIONS.split_at(split);
    [left, right]
}

pub(super) fn display_key(key: &'static str) -> &'static str {
    match (key, cfg!(target_os = "macos")) {
        ("Mod", true) => "⌘",
        ("Mod", false) => "Ctrl",
        ("Shift", true) => "⇧",
        ("Alt", true) => "⌥",
        _ => key,
    }
}
