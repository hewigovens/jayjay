use std::collections::HashMap;
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    self, RevsetDiagnostics, RevsetParseContext, RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::time_util::DatePatternContext;
use pollster::FutureExt as _;

use super::super::Repo;
use crate::types::*;

impl Repo {
    pub(crate) fn revset_workspace_context<'a>(
        &'a self,
        path_converter: &'a RepoPathUiConverter,
    ) -> RevsetWorkspaceContext<'a> {
        RevsetWorkspaceContext {
            path_converter,
            workspace_name: self.workspace_name.as_ref(),
        }
    }

    pub(crate) fn resolve_commit(
        &self,
        repo: &Arc<ReadonlyRepo>,
        rev: &str,
    ) -> CoreResult<JjCommit> {
        let settings = repo.settings();
        let aliases_map = self.revset_aliases_map(settings)?;
        let fileset_aliases_map = self.fileset_aliases_map(settings)?;
        let extensions = self.revset_extensions();
        let path_converter = self.path_converter();

        let context = RevsetParseContext {
            aliases_map: &aliases_map,
            local_variables: HashMap::new(),
            user_email: settings.user_email(),
            date_pattern_context: DatePatternContext::from(chrono::Local::now()),
            default_ignored_remote: Some(REMOTE_NAME_FOR_LOCAL_GIT_REPO),
            fileset_aliases_map: &fileset_aliases_map,
            extensions: &extensions,
            workspace: Some(self.revset_workspace_context(&path_converter)),
        };

        let mut diagnostics = RevsetDiagnostics::new();
        let expression =
            revset::parse(&mut diagnostics, rev, &context).map_err(|e| CoreError::RevNotFound {
                rev: format!("{rev}: {e}"),
            })?;

        #[allow(clippy::borrowed_box)]
        let empty_extensions: &[&Box<dyn revset::SymbolResolverExtension>] = &[];
        let symbol_resolver = SymbolResolver::new(repo.as_ref(), empty_extensions);
        let resolved = expression
            .resolve_user_expression(repo.as_ref(), &symbol_resolver)
            .map_err(|e| CoreError::RevNotFound {
                rev: format!("{rev}: {e}"),
            })?;

        let revset = resolved
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::Internal {
                message: format!("revset eval: {e}"),
            })?;

        let mut stream = revset.stream();
        let commit_id = stream
            .next()
            .block_on()
            .ok_or_else(|| CoreError::RevNotFound {
                rev: rev.to_owned(),
            })?
            .map_err(|e| CoreError::Internal {
                message: format!("revset stream: {e}"),
            })?;

        // Match jj CLI: refuse to silently pick one of several matches (e.g. a divergent change id).
        if stream.next().block_on().is_some() {
            return Err(CoreError::RevNotFound {
                rev: format!("{rev}: resolved to more than one revision"),
            });
        }

        repo.store()
            .get_commit(&commit_id)
            .map_err(|e| CoreError::Internal {
                message: format!("get commit: {e}"),
            })
    }
}
