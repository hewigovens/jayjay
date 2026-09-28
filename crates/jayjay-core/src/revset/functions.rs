/// Whether a call needs arguments; only the zero-argument form is a complete revset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Arguments {
    None,
    Required,
}

impl Arguments {
    pub(super) fn opening(self) -> &'static str {
        match self {
            Self::None => "()",
            Self::Required => "(",
        }
    }
}

/// jj's own revset functions plus the ones JayJay registers, in the order a user reads them.
pub(super) const FUNCTIONS: &[(&str, Arguments)] = &[
    ("all", Arguments::None),
    ("ancestors", Arguments::Required),
    ("at_operation", Arguments::Required),
    ("author", Arguments::Required),
    ("author_date", Arguments::Required),
    ("author_email", Arguments::Required),
    ("author_name", Arguments::Required),
    ("bisect", Arguments::Required),
    ("bookmarks", Arguments::None),
    ("builtin_immutable_heads", Arguments::None),
    ("change_id", Arguments::Required),
    ("children", Arguments::Required),
    ("coalesce", Arguments::None),
    ("commit_id", Arguments::Required),
    ("committer", Arguments::Required),
    ("committer_date", Arguments::Required),
    ("committer_email", Arguments::Required),
    ("committer_name", Arguments::Required),
    ("conflicts", Arguments::None),
    ("connected", Arguments::Required),
    ("descendants", Arguments::Required),
    ("description", Arguments::Required),
    ("diff_contains", Arguments::Required),
    ("diff_lines", Arguments::Required),
    ("diff_lines_added", Arguments::Required),
    ("diff_lines_removed", Arguments::Required),
    ("divergent", Arguments::None),
    ("empty", Arguments::None),
    ("exactly", Arguments::Required),
    ("files", Arguments::Required),
    ("first_ancestors", Arguments::Required),
    ("first_parent", Arguments::Required),
    ("fork_point", Arguments::Required),
    ("forks", Arguments::None),
    ("heads", Arguments::Required),
    ("immutable", Arguments::None),
    ("immutable_heads", Arguments::None),
    ("latest", Arguments::Required),
    ("merge_point", Arguments::Required),
    ("merges", Arguments::None),
    ("mine", Arguments::None),
    ("mutable", Arguments::None),
    ("none", Arguments::None),
    ("parents", Arguments::Required),
    ("present", Arguments::Required),
    ("reachable", Arguments::Required),
    ("remote_bookmarks", Arguments::None),
    ("remote_tags", Arguments::None),
    ("root", Arguments::None),
    ("roots", Arguments::Required),
    ("signed", Arguments::None),
    ("subject", Arguments::Required),
    ("tags", Arguments::None),
    ("tracked_remote_tags", Arguments::None),
    ("trunk", Arguments::None),
    ("untracked_remote_tags", Arguments::None),
    ("visible_heads", Arguments::None),
    ("working_copies", Arguments::None),
];

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use jj_lib::fileset::FilesetAliasesMap;
    use jj_lib::ref_name::WorkspaceNameBuf;
    use jj_lib::revset::{
        self, RevsetAliasesMap, RevsetDiagnostics, RevsetParseContext, RevsetParseError,
        RevsetParseErrorKind, RevsetWorkspaceContext,
    };
    use jj_lib::time_util::DatePatternContext;
    use jj_lib::ui_path::RepoPathUiConverter;

    use super::{Arguments, FUNCTIONS};
    use crate::repo::Repo;

    /// Parse against empty alias maps, so only the listed functions can resolve.
    fn parse(expression: &str) -> Result<(), RevsetParseError> {
        let aliases = RevsetAliasesMap::new();
        let filesets = FilesetAliasesMap::new();
        let extensions = Repo::revset_extensions();
        let paths = RepoPathUiConverter::Fs {
            cwd: PathBuf::from("."),
            base: PathBuf::from("."),
        };
        let workspace = WorkspaceNameBuf::from("default");
        let context = RevsetParseContext {
            aliases_map: &aliases,
            local_variables: HashMap::new(),
            user_email: "jayjay@example.com",
            date_pattern_context: DatePatternContext::from(chrono::Local::now()),
            default_ignored_remote: None,
            fileset_aliases_map: &filesets,
            extensions: &extensions,
            workspace: Some(RevsetWorkspaceContext {
                path_converter: &paths,
                workspace_name: workspace.as_ref(),
            }),
        };
        revset::parse(&mut RevsetDiagnostics::new(), expression, &context).map(|_| ())
    }

    #[test]
    fn every_listed_function_exists_and_zero_argument_ones_parse() {
        for (name, arguments) in FUNCTIONS {
            let probe = format!("{name}()");
            match parse(&probe) {
                Ok(()) => assert_eq!(
                    *arguments,
                    Arguments::None,
                    "{probe} parses, so it needs no arguments"
                ),
                Err(error) => assert!(
                    matches!(
                        error.kind(),
                        RevsetParseErrorKind::InvalidFunctionArguments { .. }
                    ),
                    "{probe} should report missing arguments, got {}",
                    error.kind()
                ),
            }
        }
    }
}
