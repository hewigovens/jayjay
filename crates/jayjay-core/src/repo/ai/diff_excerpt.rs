use jj_diff::{DiffSpanStyle, collapse_context_with_mapping, compute_file_diff_full_plain};

use crate::repo::Repo;
use crate::types::*;

/// Lock files drown out the real change; they stay in the stat unless nothing else changed.
const LOCK_FILE_NAMES: [&str; 4] = [
    "package-lock.json",
    "pnpm-lock.yaml",
    "go.sum",
    "Package.resolved",
];

#[derive(Debug)]
pub struct DiffExcerpt {
    pub stat: String,
    pub diff: String,
}

impl DiffExcerpt {
    /// Fits the 4K on-device window before macOS 26.4; the agent CLIs get the same.
    pub const DEFAULT_MAX_BYTES: usize = 4000;
    /// Keeps what crosses FFI bounded.
    const MAX_DIFF_BYTES: usize = 64 * 1024;

    fn new(stat: String, diff: &str) -> Self {
        Self {
            stat,
            diff: truncate(diff, Self::MAX_DIFF_BYTES),
        }
    }

    /// The stat keeps at most half of `max_bytes`; the diff gets whatever the stat left.
    pub fn text(&self, max_bytes: usize) -> String {
        let stat = truncate(&self.stat, max_bytes / 2);
        let diff = truncate(&self.diff, max_bytes.saturating_sub(stat.len()));
        format!("{stat}\n{diff}")
    }
}

impl Repo {
    /// The working copy's changes as a stat plus a unified diff for a commit-message model; lock files appear only in the stat unless they are the whole change.
    pub fn diff_excerpt(&self) -> CoreResult<Option<DiffExcerpt>> {
        self.refresh_working_copy()?;
        let hunks = self.show("@")?.diff;
        if hunks.is_empty() {
            return Ok(None);
        }
        let stat = format_stat(&self.diff_file_stats("@", false)?);
        let without_locks: Vec<&DiffHunk> = hunks
            .iter()
            .filter(|hunk| {
                !(is_lock_file(&hunk.path) && hunk.old_path.as_deref().is_none_or(is_lock_file))
            })
            .collect();
        let shown = if without_locks.is_empty() {
            hunks.iter().collect()
        } else {
            without_locks
        };
        let diff: String = shown.into_iter().map(unified_hunk).collect();
        Ok(Some(DiffExcerpt::new(stat, &diff)))
    }
}

fn is_lock_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    name.ends_with(".lock") || LOCK_FILE_NAMES.contains(&name)
}

/// `jj diff --stat` shape: one bar line per file and a totals line.
fn format_stat(files: &[FileDiffStats]) -> String {
    let mut stat = String::new();
    for file in files {
        stat.push_str(&format!(
            "{} | {} {}{}\n",
            file.path,
            file.insertions + file.deletions,
            "+".repeat(file.insertions.min(40) as usize),
            "-".repeat(file.deletions.min(40) as usize)
        ));
    }
    stat.push_str(&format!(
        "{} files changed, {} insertions(+), {} deletions(-)",
        files.len(),
        files.iter().map(|file| file.insertions).sum::<u32>(),
        files.iter().map(|file| file.deletions).sum::<u32>()
    ));
    stat
}

fn unified_hunk(hunk: &DiffHunk) -> String {
    let header = match hunk.hunk_type {
        HunkType::Added => format!("Added {}:\n", hunk.path),
        HunkType::Removed => format!("Removed {}:\n", hunk.path),
        HunkType::Modified => format!("Modified {}:\n", hunk.path),
        HunkType::Renamed => format!(
            "Renamed {} => {}:\n",
            hunk.old_path.as_deref().unwrap_or(""),
            hunk.path
        ),
    };
    if hunk.is_content_free_rename() {
        return header;
    }
    if hunk.old.content.is_none() && hunk.new.content.is_none() {
        return format!("{header}    (binary)\n");
    }
    let diff = compute_file_diff_full_plain(
        &hunk.path,
        hunk.old.content.as_deref().unwrap_or(""),
        hunk.new.content.as_deref().unwrap_or(""),
        false,
    );
    let mut text = header;
    for line in collapse_context_with_mapping(&diff).diff.lines {
        let prefix = match line.style {
            DiffSpanStyle::Added => "+",
            DiffSpanStyle::Removed => "-",
            DiffSpanStyle::Separator => "...",
            DiffSpanStyle::Context | DiffSpanStyle::Unchanged => " ",
        };
        text.push_str(prefix);
        text.push_str(&line.text());
        text.push('\n');
    }
    text
}

/// Cuts on a UTF-8 char boundary so multibyte content never panics the slice.
fn truncate(diff: &str, max_bytes: usize) -> String {
    if diff.len() <= max_bytes {
        return diff.to_owned();
    }
    let cut = (0..=max_bytes)
        .rev()
        .find(|i| diff.is_char_boundary(*i))
        .unwrap_or(0);
    format!("{}...\n(truncated)", &diff[..cut])
}

#[cfg(test)]
mod tests {
    use super::{DiffExcerpt, truncate};

    #[test]
    fn truncate_cuts_on_char_boundary() {
        assert_eq!(truncate("hi", 4000), "hi");
        assert_eq!(truncate("abc界def", 4), "abc...\n(truncated)");
        assert_eq!(truncate("abc界def", 6), "abc界...\n(truncated)");

        let emoji = format!("{}🚀tail", "x".repeat(3998));
        let out = truncate(&emoji, 4000);
        assert!(out.starts_with(&"x".repeat(3998)));
        assert!(out.ends_with("...\n(truncated)"));
        assert!(!out.contains('🚀'));
    }

    #[test]
    fn text_shares_the_budget_between_stat_and_diff() {
        let excerpt = DiffExcerpt::new("1 file changed".to_owned(), "abcdefghijklmnopqrst");
        assert_eq!(excerpt.text(100), "1 file changed\nabcdefghijklmnopqrst");
        assert_eq!(
            excerpt.text(30),
            "1 file changed\nabcdefghijklmnop...\n(truncated)"
        );

        let wide = DiffExcerpt::new("x".repeat(40), "abcdef");
        let text = wide.text(20);
        assert!(text.starts_with(&format!("{}...\n(truncated)\n", "x".repeat(10))));
        assert!(text.ends_with("(truncated)"));
        assert!(!text.contains("abcdef"));
    }
}
