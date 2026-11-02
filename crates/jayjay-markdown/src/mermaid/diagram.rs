#[cfg(feature = "mermaid")]
use super::svg;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MermaidDiagram {
    pub index: usize,
    pub width: u32,
    pub height: u32,
    source: String,
    svg: String,
}

impl MermaidDiagram {
    pub(crate) fn is_language(language: &str) -> bool {
        language
            .split_whitespace()
            .next()
            .is_some_and(|language| language.eq_ignore_ascii_case("mermaid"))
    }

    #[cfg(not(feature = "mermaid"))]
    pub(crate) fn render(_source: &str, _index: usize) -> Option<Self> {
        None
    }

    #[cfg(feature = "mermaid")]
    pub(crate) fn render(source: &str, index: usize) -> Option<Self> {
        let options = format!(
            r#"{{"idPrefix":"jayjay-mermaid-{index}","images":false,"links":false,"limit":10000,"maxEdges":500,"outputLimit":1000000}}"#
        );
        let rendered = ondine::render(source, &options, now_ms()).ok()?;
        if !rendered.width.is_finite()
            || !rendered.height.is_finite()
            || rendered.width <= 0.0
            || rendered.height <= 0.0
        {
            return None;
        }
        Some(Self {
            index,
            width: rendered.width.ceil() as u32,
            height: rendered.height.ceil() as u32,
            source: source.to_owned(),
            svg: rendered.svg,
        })
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    #[cfg(feature = "mermaid")]
    pub fn svg(&self, foreground: u32, background: u32, font_family: &str) -> String {
        let foreground = format!("#{:06x}", foreground & 0xffffff);
        let background = format!("#{:06x}", background & 0xffffff);
        svg::flatten(&self.svg, &foreground, &background, font_family)
    }

    // Unreachable without the feature, since `render` never builds a diagram.
    #[cfg(not(feature = "mermaid"))]
    pub(crate) fn html_svg(&self) -> String {
        self.svg.clone()
    }

    #[cfg(feature = "mermaid")]
    pub(crate) fn html_svg(&self) -> String {
        svg::flatten(
            &self.svg,
            "CanvasText",
            "Canvas",
            "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace",
        )
    }
}

#[cfg(all(feature = "mermaid", not(target_arch = "wasm32")))]
fn now_ms() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(f64::NAN, |elapsed| elapsed.as_secs_f64() * 1000.)
}

// SystemTime::now panics on wasm32-unknown-unknown; ondine reads a non-finite time as the epoch.
#[cfg(all(feature = "mermaid", target_arch = "wasm32"))]
fn now_ms() -> f64 {
    f64::NAN
}
