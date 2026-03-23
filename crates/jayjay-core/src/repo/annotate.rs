use super::Repo;
use crate::types::*;

impl Repo {
    /// Annotate a file: shows which revision last modified each line.
    /// Parses `jj file annotate -r <rev> <path>` output.
    pub fn annotate_file(&self, rev: &str, path: &str) -> CoreResult<Vec<AnnotationLine>> {
        let output = self.run_jj(&["file", "annotate", "-r", rev, path])?;
        let mut lines = Vec::new();

        for line in output.lines() {
            // Format: "changeId author date lineNo: text"
            // Example: "mmxlompm 360470+h 2026-03-21 10:31:24    1: # JayJay"
            let parts: Vec<&str> = line.splitn(2, ": ").collect();
            if parts.len() < 2 {
                continue;
            }

            let meta = parts[0];
            let text = parts[1].to_owned();

            // Parse metadata: "changeId author date time lineNo"
            let meta_parts: Vec<&str> = meta.split_whitespace().collect();
            if meta_parts.len() < 4 {
                continue;
            }

            let change_id = meta_parts[0].to_owned();
            let author = meta_parts[1].to_owned();
            let timestamp = format!("{} {}", meta_parts[2], meta_parts[3]);

            // Line number is the last meta part (may have leading spaces)
            let line_number = meta_parts
                .last()
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);

            lines.push(AnnotationLine {
                change_id,
                author,
                timestamp,
                line_number,
                text,
            });
        }

        Ok(lines)
    }

    /// File history: list revisions that modified a given file path.
    pub fn file_history(&self, path: &str) -> CoreResult<Vec<ChangeInfo>> {
        let revset = format!(
            "files(\"{}\")",
            path.replace('\\', "\\\\").replace('"', "\\\"")
        );
        let log = self.log(&revset)?;
        Ok(log)
    }
}
