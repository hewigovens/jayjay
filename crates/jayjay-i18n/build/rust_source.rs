use std::fmt::Write as _;

use crate::catalog::{Catalog, Locale, Visit, walk};
use crate::identifier::snake;

pub(crate) fn render(source: &[Catalog], locales: &[Locale]) -> String {
    let mut out = String::from("pub(crate) const LOCALES: &[(&str, &[&str])] = &[\n");
    for locale in locales {
        let files: Vec<String> = locale
            .paths
            .iter()
            .map(|p| format!("include_str!({:?})", p.display().to_string()))
            .collect();
        writeln!(out, "    ({:?}, &[{}]),", locale.name, files.join(", ")).unwrap();
    }
    out.push_str("];\n");
    walk(source, |visit| match visit {
        Visit::Open(depth, segment) => {
            writeln!(out, "{}pub mod {} {{", "    ".repeat(depth), snake(segment)).unwrap()
        }
        Visit::Close(depth) => writeln!(out, "{}}}", "    ".repeat(depth)).unwrap(),
        Visit::Catalog(depth, catalog) => {
            let indent = "    ".repeat(depth);
            for message in &catalog.messages {
                let mut params = Vec::new();
                let mut values = Vec::new();
                for arg in &message.args {
                    let name = snake(&arg.name);
                    let (kind, value) = if arg.number {
                        (
                            "impl Into<crate::FluentNumber>",
                            format!("crate::FluentValue::Number({name}.into())"),
                        )
                    } else {
                        ("&str", format!("crate::FluentValue::from({name})"))
                    };
                    params.push(format!("{name}: {kind}"));
                    values.push(format!("({:?}, {value})", arg.name));
                }
                let args = if values.is_empty() {
                    "None".to_owned()
                } else {
                    format!(
                        "Some(&crate::FluentArgs::from_iter([{}]))",
                        values.join(", ")
                    )
                };
                writeln!(
                    out,
                    "{indent}pub fn {}({}) -> String {{\n{indent}    crate::message({:?}, {args})\n{indent}}}",
                    snake(&message.name),
                    params.join(", "),
                    message.id
                )
                .unwrap();
            }
        }
    });
    out
}
