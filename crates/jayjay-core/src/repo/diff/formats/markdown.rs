pub(super) fn table_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\n', '\r'], " ")
}

pub(super) fn table(rows: Vec<Vec<String>>) -> String {
    let Some(first) = rows.first() else {
        return String::new();
    };

    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    write_table_row(&mut out, first, width);
    out.push('|');
    for _ in 0..width {
        out.push_str(" --- |");
    }
    out.push('\n');
    for row in rows.iter().skip(1) {
        write_table_row(&mut out, row, width);
    }
    out
}

fn write_table_row(out: &mut String, row: &[String], width: usize) {
    out.push('|');
    for index in 0..width {
        out.push(' ');
        if let Some(cell) = row.get(index) {
            out.push_str(cell);
        }
        out.push_str(" |");
    }
    out.push('\n');
}

#[cfg(test)]
mod tests {
    use super::{table, table_cell};

    #[test]
    fn escapes_markdown_table_cells() {
        assert_eq!(table_cell("a\\b|c\nd"), "a\\\\b\\|c d");
    }

    #[test]
    fn renders_rows_as_markdown_table() {
        let rows = vec![
            vec!["name".to_owned(), "value".to_owned()],
            vec!["a,b".to_owned(), "1".to_owned()],
            vec!["empty".to_owned()],
        ];

        assert_eq!(
            table(rows),
            "| name | value |\n| --- | --- |\n| a,b | 1 |\n| empty |  |\n"
        );
    }
}
