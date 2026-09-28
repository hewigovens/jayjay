pub(super) fn table_cell(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut in_line_break = false;
    for ch in value.chars() {
        match ch {
            '\\' => {
                out.push('\\');
                out.push('\\');
                in_line_break = false;
            }
            '|' => {
                out.push('\\');
                out.push('|');
                in_line_break = false;
            }
            '\n' | '\r' => {
                if !in_line_break {
                    out.push(' ');
                    in_line_break = true;
                }
            }
            _ => {
                out.push(ch);
                in_line_break = false;
            }
        }
    }
    out
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
    use super::table_cell;

    #[test]
    fn escapes_markdown_table_cells() {
        assert_eq!(table_cell("a\\b|c\r\nd"), "a\\\\b\\|c d");
    }
}
