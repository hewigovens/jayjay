/// Paste-ready jj configuration for using JayJay as a diff, diff-edit, and merge tool.
pub const JJ_TOOL_CONFIG: &str = r#"[merge-tools.jayjay]
program = "jayjay"
diff-args = ["tool", "diff", "$left", "$right"]
edit-args = ["tool", "edit", "$left", "$right"]
merge-args = ["tool", "merge", "$left", "$base", "$right", "$output", "$path", "$marker_length"]
merge-tool-edits-conflict-markers = true
"#;
