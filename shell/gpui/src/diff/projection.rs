use jayjay_core::{DiffHunk, DiffProjection, DiffProjectionMode, DiffRenderKind};

use crate::ui::icons::glyph;

pub(crate) use jayjay_core::projection::{
    cache_identity, help, opens_automatically, request_mode, shows_banner, title,
};

pub(crate) fn icon(projection: Option<&DiffProjection>) -> &'static str {
    match projection.map(|projection| projection.render_kind) {
        Some(DiffRenderKind::Text) => glyph::FILE_CODE,
        Some(DiffRenderKind::Markdown | DiffRenderKind::Table) => glyph::SPARKLE,
        None => glyph::SPARKLE,
    }
}

pub(crate) fn html_external_url(repo_path: &str, hunk: &DiffHunk) -> Option<String> {
    if hunk.projection.is_some() || !is_html_path(&hunk.path) {
        return None;
    }
    jayjay_core::repo_file_url(repo_path, &hunk.path)
}

pub(crate) fn can_render_svg_preview(hunk: &DiffHunk) -> bool {
    hunk.projection.is_none() && is_svg_path(&hunk.path)
}

pub(crate) fn can_render_markdown_file_preview(hunk: &DiffHunk) -> bool {
    hunk.projection.is_none() && is_markdown_path(&hunk.path)
}

fn can_render_projection_as_markdown(projection: Option<&DiffProjection>) -> bool {
    projection.is_some_and(|projection| {
        projection.mode == DiffProjectionMode::Processed
            && has_markdown_render_kind(Some(projection))
    })
}

pub(crate) fn has_markdown_render_kind(projection: Option<&DiffProjection>) -> bool {
    projection.is_some_and(|projection| {
        matches!(
            projection.render_kind,
            DiffRenderKind::Markdown | DiffRenderKind::Table
        )
    })
}

pub(crate) fn renders_as_markdown(path: &str, projection: Option<&DiffProjection>) -> bool {
    projection
        .map(|projection| can_render_projection_as_markdown(Some(projection)))
        .unwrap_or_else(|| is_markdown_path(path))
}

pub(crate) fn is_svg_path(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
}

fn is_markdown_path(path: &str) -> bool {
    path.rsplit('.')
        .next()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
}

fn is_html_path(path: &str) -> bool {
    let path = path.to_ascii_lowercase();
    path.ends_with(".html") || path.ends_with(".htm")
}

#[cfg(test)]
mod tests {
    use super::*;
    use jayjay_core::{DiffContent, DiffProjection, DiffProjectionMode, DiffRenderKind, HunkType};

    #[test]
    fn html_external_url_requires_html_without_projection() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("index.html"), "<html></html>").unwrap();
        let mut html_hunk = hunk("index.html");

        assert!(html_external_url(tmp.path().to_str().unwrap(), &html_hunk).is_some());

        html_hunk.projection = Some(projection("ipynb", DiffProjectionMode::Raw));
        assert_eq!(
            html_external_url(tmp.path().to_str().unwrap(), &html_hunk),
            None
        );

        let markdown = hunk("README.md");
        assert_eq!(
            html_external_url(tmp.path().to_str().unwrap(), &markdown),
            None
        );
    }

    #[test]
    fn svg_preview_requires_svg_without_projection() {
        let mut svg_hunk = hunk("logo.svg");
        assert!(can_render_svg_preview(&svg_hunk));
        assert!(is_svg_path("Assets/Logo.SVG"));

        svg_hunk.projection = Some(projection("ipynb", DiffProjectionMode::Raw));
        assert!(!can_render_svg_preview(&svg_hunk));
        assert!(!can_render_svg_preview(&hunk("logo.png")));
    }

    #[test]
    fn markdown_preview_supports_files_and_processed_projections() {
        let mut markdown_hunk = hunk("README.markdown");
        assert!(can_render_markdown_file_preview(&markdown_hunk));
        assert!(is_markdown_path("Docs/README.MD"));
        assert!(renders_as_markdown(&markdown_hunk.path, None));

        markdown_hunk.projection = Some(projection("ipynb", DiffProjectionMode::Raw));
        assert!(!can_render_markdown_file_preview(&markdown_hunk));
        assert!(has_markdown_render_kind(markdown_hunk.projection.as_ref()));
        assert!(!renders_as_markdown(
            &markdown_hunk.path,
            markdown_hunk.projection.as_ref()
        ));

        markdown_hunk.projection = Some(projection("ipynb", DiffProjectionMode::Processed));
        assert!(can_render_projection_as_markdown(
            markdown_hunk.projection.as_ref()
        ));
        assert!(renders_as_markdown(
            &markdown_hunk.path,
            markdown_hunk.projection.as_ref()
        ));
    }

    fn hunk(path: &str) -> DiffHunk {
        DiffHunk {
            path: path.to_owned(),
            old_path: None,
            old: DiffContent::default(),
            new: DiffContent::default(),
            hunk_type: HunkType::Modified,
            supports_conflict_editor: false,
            supports_file_editor: false,
            review_identity: path.to_owned(),
            projection: None,
        }
    }

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
}
