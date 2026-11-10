use crate::{DiffProjection, DiffProjectionMode};

/// Diff is a text diff of the processed content; Rendered is the post-change document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionView {
    Source,
    Diff,
    Rendered,
}

impl ProjectionView {
    pub fn default_for(projection: &DiffProjection) -> Self {
        DiffProjectionPlugin::from_projection(projection).default_view()
    }

    pub fn request_mode(self) -> DiffProjectionMode {
        match self {
            Self::Source => DiffProjectionMode::Raw,
            Self::Diff | Self::Rendered => DiffProjectionMode::Processed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffProjectionPlugin {
    Delimited,
    Ipynb,
    Plist,
    Sarif,
    Other,
}

impl DiffProjectionPlugin {
    fn from_projection(projection: &DiffProjection) -> Self {
        match projection.plugin_id.as_str() {
            "delimited" => Self::Delimited,
            "ipynb" => Self::Ipynb,
            "plist" => Self::Plist,
            "sarif" => Self::Sarif,
            _ => Self::Other,
        }
    }

    fn opens_automatically(self) -> bool {
        matches!(self, Self::Plist)
    }

    fn default_view(self) -> ProjectionView {
        match self {
            Self::Plist | Self::Ipynb => ProjectionView::Diff,
            Self::Sarif => ProjectionView::Rendered,
            Self::Delimited | Self::Other => ProjectionView::Source,
        }
    }

    fn help(self) -> String {
        match self {
            Self::Delimited => jayjay_i18n::diff::show_table_preview(),
            Self::Ipynb => jayjay_i18n::diff::show_notebook_preview(),
            Self::Plist => jayjay_i18n::diff::show_property_list_preview(),
            Self::Sarif => jayjay_i18n::diff::show_sarif_report_preview(),
            Self::Other => jayjay_i18n::diff::show_rich_preview(),
        }
    }
}

pub fn opens_automatically(projection: &DiffProjection) -> bool {
    DiffProjectionPlugin::from_projection(projection).opens_automatically()
}

pub fn default_mode(projection: Option<&DiffProjection>) -> Option<DiffProjectionMode> {
    projection.map(|projection| ProjectionView::default_for(projection).request_mode())
}

pub fn shows_source_toggle(projection: &DiffProjection) -> bool {
    DiffProjectionPlugin::from_projection(projection) == DiffProjectionPlugin::Ipynb
}

/// A conflicted or binary side would render as a broken image.
pub fn svg_opens_preview(old: &str, new: &str) -> bool {
    let mut sides = [old, new]
        .into_iter()
        .filter(|side| !side.is_empty())
        .peekable();
    sides.peek().is_some() && sides.all(|side| side.contains("<svg") && !side.contains("<<<<<<<"))
}

pub fn shows_banner(projection: &DiffProjection, view: ProjectionView) -> bool {
    !projection.diagnostics.is_empty()
        || (projection.mode == DiffProjectionMode::Processed && view != ProjectionView::Source)
}

pub fn title(projection: &DiffProjection) -> String {
    if projection.diagnostics.is_empty() {
        if DiffProjectionPlugin::from_projection(projection) == DiffProjectionPlugin::Plist {
            return jayjay_i18n::diff::binary_property_list_on_disk_previewed();
        }
        return jayjay_i18n::diff::projection_preview(&projection.plugin_label);
    }
    jayjay_i18n::diff::projection_preview_unavailable(&projection.plugin_label)
}

pub fn help(projection: Option<&DiffProjection>) -> String {
    projection
        .map(DiffProjectionPlugin::from_projection)
        .unwrap_or(DiffProjectionPlugin::Other)
        .help()
}

pub fn active_help(projection: Option<&DiffProjection>) -> String {
    if projection.is_some_and(shows_source_toggle) {
        jayjay_i18n::diff::show_notebook_diff()
    } else {
        jayjay_i18n::diff::show_source_diff()
    }
}

pub fn cache_identity(
    projection: Option<&DiffProjection>,
    mode: Option<DiffProjectionMode>,
) -> String {
    let Some(projection) = projection else {
        return "raw".to_owned();
    };
    let active_mode = mode.unwrap_or(projection.mode);
    format!(
        "{}:v{}:{}",
        projection.plugin_id,
        projection.plugin_version,
        active_mode.identity_key()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::diff_projection as projection;

    #[test]
    fn formats_open_in_their_most_reviewable_view() {
        for (plugin, view, mode, source_toggle) in [
            (
                "delimited",
                ProjectionView::Source,
                DiffProjectionMode::Raw,
                false,
            ),
            (
                "ipynb",
                ProjectionView::Diff,
                DiffProjectionMode::Processed,
                true,
            ),
            (
                "plist",
                ProjectionView::Diff,
                DiffProjectionMode::Processed,
                false,
            ),
            (
                "sarif",
                ProjectionView::Rendered,
                DiffProjectionMode::Processed,
                false,
            ),
        ] {
            let projection = projection(plugin, DiffProjectionMode::Raw);
            assert_eq!(ProjectionView::default_for(&projection), view, "{plugin}");
            assert_eq!(default_mode(Some(&projection)), Some(mode), "{plugin}");
            assert_eq!(shows_source_toggle(&projection), source_toggle, "{plugin}");
            assert_eq!(
                opens_automatically(&projection),
                plugin == "plist",
                "{plugin}"
            );
        }
    }

    #[test]
    fn svg_preview_opens_only_when_every_side_can_render() {
        let svg = "<svg xmlns=\"http://www.w3.org/2000/svg\"/>";
        let conflict = "<<<<<<< conflict 1 of 1\n<svg/>\n>>>>>>> conflict 1 of 1\n";
        for (old, new, expected) in [
            ("", svg, true),
            (svg, "", true),
            (svg, svg, true),
            (svg, conflict, false),
            ("<binary file>", svg, false),
            ("", "", false),
        ] {
            assert_eq!(svg_opens_preview(old, new), expected, "{old:?} -> {new:?}");
        }
    }

    #[test]
    fn banner_shows_for_processed_previews_and_diagnostics() {
        let raw = projection("ipynb", DiffProjectionMode::Raw);
        let processed = projection("ipynb", DiffProjectionMode::Processed);
        let plist = projection("plist", DiffProjectionMode::Processed);
        let mut failed = raw.clone();
        failed.diagnostics.push("invalid JSON".to_owned());

        assert!(!shows_banner(&raw, ProjectionView::Rendered));
        assert!(!shows_banner(&processed, ProjectionView::Source));
        assert!(shows_banner(&processed, ProjectionView::Diff));
        assert!(shows_banner(&plist, ProjectionView::Diff));
        assert!(shows_banner(&failed, ProjectionView::Source));

        assert_eq!(title(&processed), "Notebook preview");
        assert_eq!(
            title(&plist),
            "Binary property list on disk, previewed as XML"
        );
        assert_eq!(title(&failed), "Notebook preview unavailable");
    }

    #[test]
    fn cache_identity_separates_plugin_version_and_mode() {
        let mut processed = projection("ipynb", DiffProjectionMode::Processed);
        let ipynb_v1 = cache_identity(Some(&processed), None);
        let ipynb_v1_raw = cache_identity(Some(&processed), Some(DiffProjectionMode::Raw));
        processed.plugin_version = 2;
        let ipynb_v2 = cache_identity(Some(&processed), None);
        let plist = cache_identity(
            Some(&projection("plist", DiffProjectionMode::Processed)),
            None,
        );

        let keys = [
            cache_identity(None, None),
            ipynb_v1,
            ipynb_v1_raw,
            ipynb_v2,
            plist,
        ];
        let distinct: std::collections::HashSet<&String> = keys.iter().collect();
        assert_eq!(distinct.len(), keys.len(), "{keys:?}");
    }
}
