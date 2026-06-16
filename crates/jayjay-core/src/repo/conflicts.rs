use super::Repo;
use crate::types::*;

impl Repo {
    /// List conflicted files for a revision.
    pub fn resolve_list(&self, rev: &str) -> CoreResult<Vec<String>> {
        let output = self.run_jj(&["resolve", "--list", "-r", rev])?;
        Ok(output.lines().filter_map(conflict_path).collect())
    }

    /// Resolve a conflicted file using a named tool (e.g. ":ours", ":theirs", or an editor).
    pub fn resolve_with_tool(&self, rev: &str, path: &str, tool: &str) -> CoreResult<()> {
        self.run_jj_reload(&["resolve", "-r", rev, "--tool", tool, path])
    }

    /// Resolve a file by accepting "ours" (side #1).
    pub fn resolve_use_ours(&self, rev: &str, path: &str) -> CoreResult<()> {
        self.resolve_with_tool(rev, path, ":ours")
    }

    /// Resolve a file by accepting "theirs" (side #2).
    pub fn resolve_use_theirs(&self, rev: &str, path: &str) -> CoreResult<()> {
        self.resolve_with_tool(rev, path, ":theirs")
    }

    /// Read a file's content (including conflict markers) from a revision.
    pub fn file_content(&self, rev: &str, path: &str) -> CoreResult<String> {
        self.run_jj(&["file", "show", "-r", rev, path])
    }
}

/// Extract the conflicted path from a `jj resolve --list` row, splitting on the
/// `<N>-sided conflict` description rather than whitespace so paths with spaces survive.
fn conflict_path(line: &str) -> Option<String> {
    let marker = line.find("-sided conflict")?;
    // Walk left over the conflict count's digits, then the column padding.
    let before_digits = line[..marker].trim_end_matches(|c: char| c.is_ascii_digit());
    let path = before_digits.trim_end();
    (!path.is_empty()).then(|| path.to_owned())
}

#[cfg(test)]
mod tests {
    use super::conflict_path;

    #[test]
    fn keeps_spaces_in_conflicted_path() {
        assert_eq!(
            conflict_path("My Doc.txt    2-sided conflict").as_deref(),
            Some("My Doc.txt")
        );
    }

    #[test]
    fn handles_extended_conflict_description() {
        assert_eq!(
            conflict_path("f.txt    2-sided conflict including 1 deletion").as_deref(),
            Some("f.txt")
        );
    }

    #[test]
    fn ignores_rows_without_a_conflict_description() {
        assert_eq!(conflict_path(""), None);
        assert_eq!(conflict_path("   "), None);
    }
}
