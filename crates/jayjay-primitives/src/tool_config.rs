/// Paste-ready jj configuration for using JayJay as a diff, diff-edit, and merge tool.
pub const JJ_TOOL_CONFIG: &str = r#"[merge-tools.jayjay]
program = "jayjay"
diff-args = ["tool", "diff", "$left", "$right"]
edit-args = ["tool", "edit", "$left", "$right"]
merge-args = ["tool", "merge", "$left", "$base", "$right", "$output", "$path", "$marker_length"]
merge-tool-edits-conflict-markers = true
"#;

pub const JAYJAY_CONFIG_COMMAND: &str = "config";
pub const JAYJAY_REVIEW_COMMAND: &str = "review";
pub const JAYJAY_TOOL_COMMAND: &str = "tool";
