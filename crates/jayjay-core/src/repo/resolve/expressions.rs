use std::sync::Arc;

use jj_lib::op_store::RemoteRefState;
use jj_lib::revset::{
    FunctionCallNode, LoweringContext, RemoteRefSymbolExpression, RevsetDiagnostics,
    RevsetExpression, RevsetExtensions, RevsetParseError, UserRevsetExpression,
};
use jj_lib::str_util::StringExpression;

use super::super::Repo;

impl Repo {
    pub(crate) fn revset_extensions(&self) -> RevsetExtensions {
        let mut extensions = RevsetExtensions::new();
        extensions.add_custom_function("trunk", Self::trunk_revset_function);
        extensions.add_custom_function(
            "builtin_immutable_heads",
            Self::builtin_immutable_heads_revset_function,
        );
        extensions.add_custom_function("immutable_heads", Self::immutable_heads_revset_function);
        extensions.add_custom_function("immutable", Self::immutable_revset_function);
        extensions.add_custom_function("mutable", Self::mutable_revset_function);
        extensions
    }

    fn trunk_revset_function(
        _diagnostics: &mut RevsetDiagnostics,
        function: &FunctionCallNode,
        _context: &LoweringContext,
    ) -> Result<Arc<UserRevsetExpression>, RevsetParseError> {
        function.expect_no_arguments()?;
        Ok(Self::trunk_expression())
    }

    fn builtin_immutable_heads_revset_function(
        _diagnostics: &mut RevsetDiagnostics,
        function: &FunctionCallNode,
        _context: &LoweringContext,
    ) -> Result<Arc<UserRevsetExpression>, RevsetParseError> {
        function.expect_no_arguments()?;
        Ok(Self::immutable_heads_expression())
    }

    fn immutable_heads_revset_function(
        _diagnostics: &mut RevsetDiagnostics,
        function: &FunctionCallNode,
        _context: &LoweringContext,
    ) -> Result<Arc<UserRevsetExpression>, RevsetParseError> {
        function.expect_no_arguments()?;
        Ok(Self::immutable_heads_expression())
    }

    fn immutable_revset_function(
        _diagnostics: &mut RevsetDiagnostics,
        function: &FunctionCallNode,
        _context: &LoweringContext,
    ) -> Result<Arc<UserRevsetExpression>, RevsetParseError> {
        function.expect_no_arguments()?;
        Ok(Self::immutable_expression())
    }

    fn mutable_revset_function(
        _diagnostics: &mut RevsetDiagnostics,
        function: &FunctionCallNode,
        _context: &LoweringContext,
    ) -> Result<Arc<UserRevsetExpression>, RevsetParseError> {
        function.expect_no_arguments()?;
        Ok(RevsetExpression::all().minus(&Self::immutable_expression()))
    }

    fn trunk_expression() -> Arc<UserRevsetExpression> {
        let candidates = [
            Self::remote_bookmark_expression("main", "origin"),
            Self::remote_bookmark_expression("master", "origin"),
            Self::remote_bookmark_expression("trunk", "origin"),
            Self::remote_bookmark_expression("main", "upstream"),
            Self::remote_bookmark_expression("master", "upstream"),
            Self::remote_bookmark_expression("trunk", "upstream"),
            RevsetExpression::root(),
        ];
        RevsetExpression::union_all(&candidates).latest(1)
    }

    fn immutable_heads_expression() -> Arc<UserRevsetExpression> {
        let any_string = StringExpression::all();
        let any_remote = StringExpression::all();
        RevsetExpression::union_all(&[
            Self::trunk_expression(),
            RevsetExpression::tags(any_string.clone()),
            RevsetExpression::remote_bookmarks(
                RemoteRefSymbolExpression {
                    name: any_string,
                    remote: any_remote,
                },
                Some(RemoteRefState::New),
            ),
        ])
    }

    fn immutable_expression() -> Arc<UserRevsetExpression> {
        RevsetExpression::union_all(&[Self::immutable_heads_expression(), RevsetExpression::root()])
            .ancestors()
    }

    fn remote_bookmark_expression(name: &str, remote: &str) -> Arc<UserRevsetExpression> {
        RevsetExpression::remote_bookmarks(
            RemoteRefSymbolExpression {
                name: StringExpression::exact(name),
                remote: StringExpression::exact(remote),
            },
            None,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use jj_lib::fileset::FilesetAliasesMap;
    use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
    use jj_lib::revset::{self, RevsetAliasesMap, RevsetDiagnostics, RevsetParseContext};
    use jj_test::init_jj_repo;

    use super::*;

    #[test]
    fn immutable_heads_alias_can_use_builtin_function_without_cli_aliases() {
        let temp_dir = init_jj_repo();
        let repo_path = temp_dir.path().join("repo");
        let repo = Repo::open(&repo_path).expect("open repo");
        let path_converter = repo.path_converter();
        let workspace = repo.revset_workspace_context(&path_converter);

        let mut aliases_map = RevsetAliasesMap::new();
        aliases_map
            .insert(
                "immutable_heads()",
                "builtin_immutable_heads()".to_owned(),
                None,
            )
            .expect("insert alias");
        let fileset_aliases_map = FilesetAliasesMap::new();
        let extensions = repo.revset_extensions();
        let context = RevsetParseContext {
            aliases_map: &aliases_map,
            local_variables: HashMap::new(),
            user_email: "",
            date_pattern_context: chrono::Local::now().into(),
            default_ignored_remote: Some(REMOTE_NAME_FOR_LOCAL_GIT_REPO),
            fileset_aliases_map: &fileset_aliases_map,
            use_glob_by_default: true,
            extensions: &extensions,
            workspace: Some(workspace),
        };

        revset::parse(
            &mut RevsetDiagnostics::new(),
            "present(@) | ancestors(immutable_heads().., 20) | trunk()",
            &context,
        )
        .expect("parse immutable_heads alias");
    }
}
