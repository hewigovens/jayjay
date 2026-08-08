use crate::{DiffProjection, DiffProjectionMode};

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

    fn help(self) -> &'static str {
        match self {
            Self::Delimited => "Show table preview",
            Self::Ipynb => "Show notebook preview",
            Self::Plist => "Show property list preview",
            Self::Sarif => "Show SARIF report preview",
            Self::Other => "Show rich preview",
        }
    }
}

pub fn opens_automatically(projection: &DiffProjection) -> bool {
    DiffProjectionPlugin::from_projection(projection).opens_automatically()
}

pub fn request_mode(
    projection: Option<&DiffProjection>,
    rich_view: bool,
) -> Option<DiffProjectionMode> {
    let projection = projection?;
    if opens_automatically(projection) {
        return Some(DiffProjectionMode::Processed);
    }
    Some(if rich_view {
        DiffProjectionMode::Processed
    } else {
        DiffProjectionMode::Raw
    })
}

pub fn shows_banner(projection: &DiffProjection, rich_view: bool) -> bool {
    !projection.diagnostics.is_empty()
        || (projection.mode == DiffProjectionMode::Processed
            && (rich_view || opens_automatically(projection)))
}

pub fn title(projection: &DiffProjection) -> String {
    if projection.diagnostics.is_empty() {
        if DiffProjectionPlugin::from_projection(projection) == DiffProjectionPlugin::Plist {
            return "Binary property list on disk, previewed as XML".to_owned();
        }
        return format!("{} preview", projection.plugin_label);
    }
    format!("{} preview unavailable", projection.plugin_label)
}

pub fn help(projection: Option<&DiffProjection>) -> &'static str {
    projection
        .map(DiffProjectionPlugin::from_projection)
        .unwrap_or(DiffProjectionPlugin::Other)
        .help()
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
    use crate::DiffRenderKind;

    fn projection(plugin_id: &str, mode: DiffProjectionMode) -> DiffProjection {
        DiffProjection {
            plugin_id: plugin_id.to_owned(),
            plugin_label: "Notebook".to_owned(),
            plugin_version: 1,
            mode,
            render_kind: DiffRenderKind::Markdown,
            virtual_path: "analysis.ipynb.md".to_owned(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn plist_requests_processed_mode_without_user_toggle() {
        let projection = projection("plist", DiffProjectionMode::Raw);

        assert_eq!(
            request_mode(Some(&projection), false),
            Some(DiffProjectionMode::Processed)
        );
    }

    #[test]
    fn rich_projection_toggle_requests_processed_mode() {
        let projection = projection("ipynb", DiffProjectionMode::Raw);

        assert_eq!(
            request_mode(Some(&projection), false),
            Some(DiffProjectionMode::Raw)
        );
        assert_eq!(
            request_mode(Some(&projection), true),
            Some(DiffProjectionMode::Processed)
        );
    }

    #[test]
    fn plugin_ids_map_to_reusable_projection_plugin_kinds() {
        assert_eq!(
            DiffProjectionPlugin::from_projection(&projection(
                "delimited",
                DiffProjectionMode::Raw
            )),
            DiffProjectionPlugin::Delimited
        );
        assert_eq!(
            DiffProjectionPlugin::from_projection(&projection("ipynb", DiffProjectionMode::Raw)),
            DiffProjectionPlugin::Ipynb
        );
        assert_eq!(
            DiffProjectionPlugin::from_projection(&projection("plist", DiffProjectionMode::Raw)),
            DiffProjectionPlugin::Plist
        );
        assert_eq!(
            DiffProjectionPlugin::from_projection(&projection("sarif", DiffProjectionMode::Raw)),
            DiffProjectionPlugin::Sarif
        );
    }
}
