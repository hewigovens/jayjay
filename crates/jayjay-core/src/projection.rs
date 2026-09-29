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
    use crate::mock::diff_projection as projection;

    #[test]
    fn rich_projection_toggle_requests_processed_mode() {
        for (plugin, rich_view, expected) in [
            ("ipynb", false, DiffProjectionMode::Raw),
            ("ipynb", true, DiffProjectionMode::Processed),
            ("plist", false, DiffProjectionMode::Processed),
            ("plist", true, DiffProjectionMode::Processed),
        ] {
            let projection = projection(plugin, DiffProjectionMode::Raw);
            assert_eq!(
                request_mode(Some(&projection), rich_view),
                Some(expected),
                "{plugin}, rich_view={rich_view}"
            );
        }
    }

    #[test]
    fn banner_shows_for_processed_previews_and_diagnostics() {
        let raw = projection("ipynb", DiffProjectionMode::Raw);
        let processed = projection("ipynb", DiffProjectionMode::Processed);
        let plist = projection("plist", DiffProjectionMode::Processed);
        let mut failed = raw.clone();
        failed.diagnostics.push("invalid JSON".to_owned());

        assert!(!shows_banner(&raw, true));
        assert!(!shows_banner(&processed, false));
        assert!(shows_banner(&processed, true));
        assert!(shows_banner(&plist, false));
        assert!(shows_banner(&failed, false));

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
