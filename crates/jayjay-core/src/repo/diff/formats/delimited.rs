use csv::ReaderBuilder;

use crate::types::*;

use super::{
    FormatInput, ProjectionPair, markdown,
    types::{DiffFormatPlugin, has_extension, project_text_pair, projection_error},
};

pub(super) struct DelimitedPlugin;

impl DiffFormatPlugin for DelimitedPlugin {
    fn id(&self) -> &'static str {
        "delimited"
    }

    fn version(&self) -> u32 {
        1
    }

    fn label(&self) -> &'static str {
        "Delimited table"
    }

    fn render_kind(&self) -> DiffRenderKind {
        DiffRenderKind::Table
    }

    fn matches_path(&self, path: &str) -> bool {
        has_extension(path, &["csv", "tsv"])
    }

    fn virtual_path(&self, path: &str) -> String {
        format!("{path}.md")
    }

    fn project(&self, input: FormatInput<'_>) -> CoreResult<ProjectionPair> {
        let delimiter = delimiter_for_path(input.path);
        project_text_pair(
            input,
            self.projection(input.path, DiffProjectionMode::Processed),
            |bytes| project_table(bytes, delimiter),
        )
    }
}

fn delimiter_for_path(path: &str) -> u8 {
    if has_extension(path, &["tsv"]) {
        b'\t'
    } else {
        b','
    }
}

fn project_table(bytes: &[u8], delimiter: u8) -> CoreResult<String> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .delimiter(delimiter)
        .from_reader(bytes);
    let mut rows = Vec::new();

    for record in reader.byte_records() {
        let record = record.map_err(|err| projection_error(format!("parse table: {err}")))?;
        let row = record
            .iter()
            .map(parsed_table_cell)
            .collect::<CoreResult<Vec<_>>>()?;
        rows.push(row);
    }

    Ok(markdown::table(rows))
}

fn parsed_table_cell(bytes: &[u8]) -> CoreResult<String> {
    let value = std::str::from_utf8(bytes)
        .map_err(|err| projection_error(format!("table projection is not UTF-8: {err}")))?;
    Ok(markdown::table_cell(value))
}

#[cfg(test)]
mod tests {
    use super::project_table;

    #[test]
    fn projects_csv_as_markdown_table() {
        let projected =
            project_table(b"name,value\r\n\"a,b\",1\r\nempty,\r\n", b',').expect("project csv");

        assert_eq!(
            projected,
            "| name | value |\n| --- | --- |\n| a,b | 1 |\n| empty |  |\n"
        );
    }
}
