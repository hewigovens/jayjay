use serde_json::Value;

use crate::types::*;

use super::{
    FormatInput, ProjectionPair,
    types::{DiffFormatPlugin, has_extension, project_text_pair, projection_error},
};

pub(super) struct IpynbPlugin;

impl DiffFormatPlugin for IpynbPlugin {
    fn id(&self) -> &'static str {
        "ipynb"
    }

    fn version(&self) -> u32 {
        1
    }

    fn label(&self) -> &'static str {
        "Notebook"
    }

    fn render_kind(&self) -> DiffRenderKind {
        DiffRenderKind::Markdown
    }

    fn matches_path(&self, path: &str) -> bool {
        has_extension(path, &["ipynb"])
    }

    fn virtual_path(&self, path: &str) -> String {
        format!("{path}.md")
    }

    fn project(&self, input: FormatInput<'_>) -> CoreResult<ProjectionPair> {
        project_text_pair(
            input,
            self.projection(input.path, DiffProjectionMode::Processed),
            project_notebook,
        )
    }
}

fn project_notebook(bytes: &[u8]) -> CoreResult<String> {
    let notebook: Value = serde_json::from_slice(bytes)
        .map_err(|err| projection_error(format!("parse notebook JSON: {err}")))?;
    let cells = notebook
        .get("cells")
        .and_then(Value::as_array)
        .ok_or_else(|| projection_error("notebook has no cells array"))?;
    let language = notebook_language(&notebook);
    let mut out = String::new();

    for (index, cell) in cells.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        let cell_type = cell
            .get("cell_type")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        match cell_type {
            "markdown" => {
                out.push_str(&source_text(cell));
            }
            "code" => {
                out.push_str("```");
                out.push_str(language);
                out.push('\n');
                out.push_str(&source_text(cell));
                ensure_trailing_newline(&mut out);
                out.push_str("```\n");
            }
            _ => {
                out.push_str(&source_text(cell));
            }
        }
        ensure_trailing_newline(&mut out);
    }

    Ok(out)
}

fn notebook_language(notebook: &Value) -> &'static str {
    let language = notebook
        .pointer("/metadata/language_info/name")
        .or_else(|| notebook.pointer("/metadata/kernelspec/language"))
        .and_then(Value::as_str)
        .unwrap_or("");
    match language.to_ascii_lowercase().as_str() {
        "python" => "python",
        "javascript" => "javascript",
        "typescript" => "typescript",
        "r" => "r",
        "rust" => "rust",
        "swift" => "swift",
        _ => "",
    }
}

fn source_text(cell: &Value) -> String {
    match cell.get("source") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(lines)) => lines.iter().filter_map(Value::as_str).collect(),
        _ => String::new(),
    }
}

fn ensure_trailing_newline(text: &mut String) {
    if !text.ends_with('\n') {
        text.push('\n');
    }
}

#[cfg(test)]
mod tests {
    use super::project_notebook;

    #[test]
    fn projects_notebook_cells_without_outputs_or_execution_counts() {
        let projected = project_notebook(
            br##"{
              "metadata": {"language_info": {"name": "python"}},
              "cells": [
                {"cell_type": "markdown", "source": ["# Title\n"], "metadata": {}},
                {"cell_type": "code", "execution_count": 12, "source": ["print(1)\n"], "outputs": [{"text": "volatile"}]}
              ]
            }"##,
        )
        .expect("project notebook");

        assert!(projected.contains("# Title"));
        assert!(projected.contains("```python\nprint(1)\n```"));
        assert!(!projected.contains("execution_count"));
        assert!(!projected.contains("volatile"));
    }
}
