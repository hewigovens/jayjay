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
