use super::Repo;
use crate::types::*;

impl Repo {
    /// List conflicted files for a revision.
    pub fn resolve_list(&self, rev: &str) -> CoreResult<Vec<String>> {
        let output = self.run_jj(&["resolve", "--list", "-r", rev])?;
        let files: Vec<String> = output
            .lines()
            .filter_map(|line| {
                // Format: "path    2-sided conflict"
                let path = line.split_whitespace().next()?;
                if path.is_empty() {
                    None
                } else {
                    Some(path.to_owned())
                }
            })
            .collect();
        Ok(files)
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
