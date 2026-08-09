use std::path::Path;

use crate::filesystem::normalized_absolute_path;
use crate::{CoreError, CoreResult};

use super::JJ_INSTRUCTIONS;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExternalToolInvocation {
    Diff {
        left: String,
        right: String,
        editable: bool,
    },
    Merge {
        left: String,
        base: String,
        right: String,
        output: String,
        path: String,
        marker_length: u32,
    },
}

impl ExternalToolInvocation {
    pub fn title(&self) -> String {
        match self {
            Self::Diff { editable: true, .. } => "JayJay — Edit Diff".to_owned(),
            Self::Diff { .. } => "JayJay — Compare".to_owned(),
            Self::Merge { path, .. } => format!("JayJay — Resolve {path}"),
        }
    }

    pub fn cancel_exit_code(&self) -> i32 {
        match self {
            Self::Diff { editable: true, .. } | Self::Merge { .. } => 1,
            Self::Diff { .. } => 0,
        }
    }
}

pub fn parse_external_tool_invocation(
    arguments: &[String],
) -> CoreResult<Option<ExternalToolInvocation>> {
    if arguments.len() == 2 {
        let left = normalized_absolute_path(&arguments[0]);
        let right = normalized_absolute_path(&arguments[1]);
        let editable = Path::new(&right).join(JJ_INSTRUCTIONS).is_file();
        return Ok(Some(ExternalToolInvocation::Diff {
            left,
            right,
            editable,
        }));
    }
    if arguments.first().map(String::as_str) != Some("tool") {
        return Ok(None);
    }
    match arguments.get(1).map(String::as_str) {
        Some(mode @ ("diff" | "edit")) if arguments.len() == 4 => {
            Ok(Some(ExternalToolInvocation::Diff {
                left: normalized_absolute_path(&arguments[2]),
                right: normalized_absolute_path(&arguments[3]),
                editable: mode == "edit",
            }))
        }
        Some("merge") if matches!(arguments.len(), 6 | 8) => {
            let output = normalized_absolute_path(&arguments[5]);
            let path = arguments.get(6).cloned().unwrap_or_else(|| {
                Path::new(&output)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("merge result")
                    .to_owned()
            });
            Ok(Some(ExternalToolInvocation::Merge {
                left: normalized_absolute_path(&arguments[2]),
                base: if arguments[3].is_empty() {
                    String::new()
                } else {
                    normalized_absolute_path(&arguments[3])
                },
                right: normalized_absolute_path(&arguments[4]),
                output,
                path,
                marker_length: arguments
                    .get(7)
                    .and_then(|length| length.parse().ok())
                    .unwrap_or(7),
            }))
        }
        Some("diff" | "edit") => Err(CoreError::internal(
            "usage: jayjay tool diff|edit <left> <right>",
        )),
        Some("merge") => Err(CoreError::internal(
            "usage: jayjay tool merge <left> <base> <right> <output> [<path> <marker-length>]",
        )),
        Some(mode) => Err(CoreError::Internal {
            message: format!("unknown tool mode: {mode}"),
        }),
        None => Err(CoreError::Internal {
            message: "missing tool mode".to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn parses_explicit_diff_edit_and_merge() {
        assert!(matches!(
            parse_external_tool_invocation(&args(&["tool", "diff", "left", "right"])),
            Ok(Some(ExternalToolInvocation::Diff {
                editable: false,
                ..
            }))
        ));
        assert!(matches!(
            parse_external_tool_invocation(&args(&[
                "tool", "merge", "local", "base", "remote", "merged.rs",
            ])),
            Ok(Some(ExternalToolInvocation::Merge {
                path,
                marker_length: 7,
                ..
            })) if path == "merged.rs"
        ));
        assert!(matches!(
            parse_external_tool_invocation(&args(&["tool", "edit", "left", "right"])),
            Ok(Some(ExternalToolInvocation::Diff { editable: true, .. }))
        ));
        assert!(matches!(
            parse_external_tool_invocation(&args(&[
                "tool",
                "merge",
                "left",
                "base",
                "right",
                "output",
                "src/lib.rs",
                "11",
            ])),
            Ok(Some(ExternalToolInvocation::Merge {
                marker_length: 11,
                ..
            }))
        ));
    }

    #[test]
    fn non_tool_repo_path_falls_through() {
        assert_eq!(
            parse_external_tool_invocation(&args(&["/repo"])).unwrap(),
            None
        );
    }

    #[test]
    fn plain_two_paths_detect_diff_edit_instructions() {
        let left = tempfile::tempdir().expect("left");
        let right = tempfile::tempdir().expect("right");
        std::fs::write(right.path().join(JJ_INSTRUCTIONS), "instructions").expect("instructions");
        let arguments = [
            left.path().to_string_lossy().into_owned(),
            right.path().to_string_lossy().into_owned(),
        ];

        assert!(matches!(
            parse_external_tool_invocation(&arguments),
            Ok(Some(ExternalToolInvocation::Diff { editable: true, .. }))
        ));
    }

    #[test]
    fn malformed_tool_invocations_return_usage_errors() {
        let error = parse_external_tool_invocation(&args(&["tool", "merge", "left"]))
            .expect_err("invalid merge invocation");

        assert!(error.to_string().contains("usage: jayjay tool merge"));
    }
}
