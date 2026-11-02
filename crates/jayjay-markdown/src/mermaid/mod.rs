mod diagram;
#[cfg(feature = "mermaid")]
mod svg;

pub use diagram::MermaidDiagram;

#[cfg(all(test, feature = "mermaid"))]
mod tests;
