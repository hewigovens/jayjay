use gpui::AnyElement;

use super::chrome::section_header;
use crate::app::theme::Theme;

pub(crate) trait PickerRow {
    type Action: Clone;

    fn action(&self) -> Option<Self::Action>;
}

pub(crate) struct PickerSection<R> {
    pub(crate) id: &'static str,
    pub(crate) title: Option<&'static str>,
    pub(crate) rows: Vec<R>,
    best_score: u32,
}

impl<R> PickerSection<R> {
    pub(crate) fn filtered(
        id: &'static str,
        title: Option<&'static str>,
        rows: Vec<R>,
        query: &str,
        search_text: impl Fn(&R) -> String,
    ) -> Option<Self> {
        let search_texts = rows.iter().map(search_text).collect::<Vec<_>>();
        let ranked = jayjay_core::fuzzy::rank_scored(query, &search_texts);
        let best_score = ranked.first().map_or(0, |(_, score)| *score);
        let mut slots = rows.into_iter().map(Some).collect::<Vec<_>>();
        let rows = ranked
            .into_iter()
            .filter_map(|(index, _)| slots[index as usize].take())
            .collect::<Vec<_>>();
        (!rows.is_empty()).then_some(Self {
            id,
            title,
            rows,
            best_score,
        })
    }
}

/// Sections in declared order for an empty query; with a query, the section holding the best match comes first so selection zero is the global best.
pub(crate) fn sections_by_best_match<R>(
    sections: impl IntoIterator<Item = PickerSection<R>>,
) -> Vec<PickerSection<R>> {
    let mut sections = sections.into_iter().collect::<Vec<_>>();
    sections.sort_by_key(|section| std::cmp::Reverse(section.best_score));
    sections
}

/// Pairs each actionable row with its list item index (headers count) so the selection can scroll to it.
pub(crate) fn picker_actions<R: PickerRow>(
    sections: &[PickerSection<R>],
) -> Vec<(R::Action, usize)> {
    let mut item_index = 0;
    let mut actions = Vec::new();
    for section in sections {
        item_index += usize::from(section.title.is_some());
        for row in &section.rows {
            if let Some(action) = row.action() {
                actions.push((action, item_index));
            }
            item_index += 1;
        }
    }
    actions
}

pub(crate) fn render_sections<R: PickerRow>(
    sections: Vec<PickerSection<R>>,
    selected: Option<usize>,
    t: &Theme,
    mut render_row: impl FnMut(R, bool) -> AnyElement,
) -> Vec<AnyElement> {
    let mut action_index = 0;
    let mut elements = Vec::new();
    for section in sections {
        if let Some(title) = section.title {
            elements.push(section_header(section.id, title, t));
        }
        for row in section.rows {
            let actionable = row.action().is_some();
            let is_selected = actionable && selected == Some(action_index);
            action_index += usize::from(actionable);
            elements.push(render_row(row, is_selected));
        }
    }
    elements
}

#[cfg(test)]
mod tests {
    use super::{PickerRow, PickerSection, picker_actions, sections_by_best_match};

    struct Row(Option<&'static str>);

    impl PickerRow for Row {
        type Action = &'static str;

        fn action(&self) -> Option<&'static str> {
            self.0
        }
    }

    #[test]
    fn action_indices_skip_headers_and_inert_rows() {
        let sections = vec![
            PickerSection {
                id: "workspaces",
                title: Some("Workspaces"),
                rows: vec![Row(None), Row(Some("open workspace"))],
                best_score: 0,
            },
            PickerSection {
                id: "global",
                title: None,
                rows: vec![Row(Some("repository list"))],
                best_score: 0,
            },
        ];

        assert_eq!(
            picker_actions(&sections),
            vec![("open workspace", 2), ("repository list", 3)]
        );
    }

    #[test]
    fn filtered_section_drops_non_matching_rows_and_empty_sections() {
        let rows = || vec![Row(Some("alpha")), Row(Some("beta"))];
        let section = PickerSection::filtered("s", None, rows(), "alp", |row| {
            row.0.unwrap_or_default().to_owned()
        })
        .expect("alpha matches");
        assert_eq!(section.rows.len(), 1);
        let ranked = PickerSection::filtered(
            "s",
            None,
            vec![Row(Some("domain")), Row(Some("main"))],
            "main",
            |row| row.0.unwrap_or_default().to_owned(),
        )
        .expect("both match");
        assert_eq!(
            ranked.rows.iter().map(|row| row.0).collect::<Vec<_>>(),
            [Some("main"), Some("domain")],
            "the best match comes first so Enter activates it"
        );
        assert!(
            PickerSection::filtered("s", None, rows(), "zzz", |row| row
                .0
                .unwrap_or_default()
                .to_owned())
            .is_none()
        );
        assert_eq!(
            PickerSection::filtered("s", None, rows(), "", |row| row
                .0
                .unwrap_or_default()
                .to_owned())
            .map(|section| section.rows.len()),
            Some(2)
        );
    }

    #[test]
    fn the_section_with_the_best_match_comes_first_only_for_a_query() {
        let text = |row: &Row| row.0.unwrap_or_default().to_owned();
        let build = |query: &str| {
            sections_by_best_match(
                [
                    PickerSection::filtered(
                        "tracked",
                        None,
                        vec![Row(Some("domain"))],
                        query,
                        text,
                    ),
                    PickerSection::filtered("local", None, vec![Row(Some("main"))], query, text),
                ]
                .into_iter()
                .flatten(),
            )
            .into_iter()
            .map(|section| section.id)
            .collect::<Vec<_>>()
        };
        assert_eq!(build(""), ["tracked", "local"]);
        assert_eq!(build("main"), ["local", "tracked"]);
    }
}
