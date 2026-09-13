use jayjay_core::load_jj_user_config;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JjConfigSnapshot {
    pub(super) path: String,
    pub(super) sections: Vec<JjConfigSection>,
    pub(super) error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JjConfigSection {
    pub(super) name: String,
    pub(super) entries: Vec<JjConfigEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct JjConfigEntry {
    pub(super) key: String,
    pub(super) value: String,
}

/// Re-reads user config files on every call — the caller (`ensure_jj_config_loaded`)
/// already caches the result per `SettingsView` instance, so this must stay fresh
/// rather than memoizing for the process lifetime.
pub(super) fn load_jj_config_snapshot() -> JjConfigSnapshot {
    load_jj_config()
}

fn load_jj_config() -> JjConfigSnapshot {
    let snapshot = load_jj_user_config();
    let sections = parse_config_sections(&snapshot.listing);
    let error = snapshot.error.or_else(|| {
        (snapshot.path.is_empty() && sections.is_empty()).then(|| "Config not found".to_owned())
    });
    JjConfigSnapshot {
        path: snapshot.path,
        sections,
        error,
    }
}

/// Groups by section name across the whole file, not just adjacent lines — the
/// flattened listing is not grouped, so the same section commonly reappears
/// non-contiguously (e.g. `ui.editor` then other sections then `ui.diff`).
pub(super) fn parse_config_sections(raw: &str) -> Vec<JjConfigSection> {
    let mut order: Vec<String> = Vec::new();
    let mut by_name: std::collections::HashMap<String, Vec<JjConfigEntry>> = Default::default();

    for line in raw.lines() {
        let Some((full_key, value)) = line.split_once(" = ") else {
            continue;
        };
        let full_key = full_key.trim();
        let value = value.trim();
        let (section, key) = full_key.split_once('.').unwrap_or(("general", full_key));
        let entries = by_name.entry(section.to_owned()).or_insert_with(|| {
            order.push(section.to_owned());
            Vec::new()
        });
        entries.push(JjConfigEntry {
            key: key.to_owned(),
            value: value.to_owned(),
        });
    }

    order
        .into_iter()
        .map(|name| {
            let entries = by_name.remove(&name).unwrap_or_default();
            JjConfigSection { name, entries }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_config_sections;

    #[test]
    fn parse_config_sections_groups_by_prefix() {
        let sections = parse_config_sections(
            "user.name = Alice\nuser.email = a@example.com\nui.diff = split\n",
        );

        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].name, "user");
        assert_eq!(sections[0].entries[0].key, "name");
        assert_eq!(sections[0].entries[1].value, "a@example.com");
        assert_eq!(sections[1].name, "ui");
        assert_eq!(sections[1].entries[0].value, "split");
    }

    #[test]
    fn parse_config_sections_keeps_equals_inside_key() {
        let sections = parse_config_sections("remotes.foo=bar.auto-track-bookmarks = \"glob:*\"\n");

        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].name, "remotes");
        assert_eq!(sections[0].entries[0].key, "foo=bar.auto-track-bookmarks");
        assert_eq!(sections[0].entries[0].value, "\"glob:*\"");
    }

    #[test]
    fn parse_config_sections_merges_non_contiguous_occurrences() {
        // The flattened listing isn't grouped, so the same section can reappear
        // after other sections — those entries must land in one merged group.
        let sections = parse_config_sections(
            "operation.hostname = host\nui.editor = code\nuser.name = Alice\nui.diff = split\n",
        );

        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].name, "operation");
        assert_eq!(sections[1].name, "ui");
        assert_eq!(sections[1].entries.len(), 2);
        assert_eq!(sections[1].entries[0].key, "editor");
        assert_eq!(sections[1].entries[1].key, "diff");
        assert_eq!(sections[2].name, "user");
    }
}
