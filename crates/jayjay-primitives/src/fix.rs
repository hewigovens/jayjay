use std::collections::{BTreeMap, BTreeSet};

const SHOWN_STDERR_LINES: usize = 3;
const SHOWN_LINE_CHARS: usize = 200;

/// What rewriting changes through the configured `[fix.tools]` did.
#[derive(Debug, Clone, Default)]
pub struct FixSummary {
    pub checked_changes: u32,
    pub fixed_changes: u32,
    pub failures: Vec<FixToolFailure>,
}

impl FixSummary {
    /// The counts, then one line per failed tool: the first file it failed on and the start of that stderr.
    pub fn message(&self) -> String {
        let mut by_tool: BTreeMap<&str, Vec<&FixToolFailure>> = BTreeMap::new();
        for failure in &self.failures {
            by_tool.entry(&failure.tool).or_default().push(failure);
        }
        let noun = if self.checked_changes == 1 {
            "change"
        } else {
            "changes"
        };
        let mut message = format!(
            "Formatters rewrote {} of {} {noun}",
            self.fixed_changes, self.checked_changes
        );
        for (tool, failures) in by_tool {
            let first = failures[0];
            let paths: BTreeSet<&str> = failures
                .iter()
                .map(|failure| failure.path.as_str())
                .collect();
            let more = match paths.len() - 1 {
                0 => String::new(),
                count => format!(" and {count} more"),
            };
            let mut lines = first.message.lines();
            let mut reason = lines
                .by_ref()
                .take(SHOWN_STDERR_LINES)
                .map(clip)
                .collect::<Vec<_>>()
                .join("\n");
            if lines.next().is_some() {
                reason.push_str("\n…");
            }
            message.push_str(&format!(
                "\n{tool} failed on {}{more}: {reason}",
                first.path
            ));
        }
        message
    }
}

fn clip(line: &str) -> String {
    match line.char_indices().nth(SHOWN_LINE_CHARS) {
        Some((end, _)) => format!("{}…", &line[..end]),
        None => line.to_owned(),
    }
}

/// A `[fix.tools.<name>]` run that could not transform one file, which keeps its old content.
#[derive(Debug, Clone)]
pub struct FixToolFailure {
    pub tool: String,
    pub path: String,
    /// The tool's stderr, or how it failed when it wrote none.
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{FixSummary, FixToolFailure};

    fn failure(tool: &str, path: &str, message: &str) -> FixToolFailure {
        FixToolFailure {
            tool: tool.to_owned(),
            path: path.to_owned(),
            message: message.to_owned(),
        }
    }

    #[test]
    fn message_reports_one_bounded_line_per_failed_tool() {
        let summary = FixSummary {
            checked_changes: 3,
            fixed_changes: 2,
            failures: vec![
                failure("sorter", "src/a.rs", "exit status: 1"),
                failure(
                    "fmt",
                    "src/b.rs",
                    "error: expected `;`\n --> 3:1\n  |\n3 | }",
                ),
                failure("sorter", "src/c.rs", "exit status: 1"),
                failure("sorter", "src/c.rs", "exit status: 1"),
                failure("lint", "src/d.rs", &"x".repeat(300)),
            ],
        };

        assert_eq!(
            summary.message(),
            format!(
                "Formatters rewrote 2 of 3 changes\n\
                 fmt failed on src/b.rs: error: expected `;`\n --> 3:1\n  |\n…\n\
                 lint failed on src/d.rs: {}…\n\
                 sorter failed on src/a.rs and 1 more: exit status: 1",
                "x".repeat(200)
            )
        );
    }
}
