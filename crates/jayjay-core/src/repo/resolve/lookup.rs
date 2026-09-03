use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{self, RevsetWorkspaceContext, SymbolResolver};

use super::super::Repo;
use super::super::support::block_on;
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
        let expression = self
            .parse_revset(
                &aliases_map,
                &fileset_aliases_map,
                settings.user_email(),
                rev,
            )
            .map_err(|e| CoreError::RevNotFound {
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
        let commit_id = block_on(stream.next())
            .ok_or_else(|| CoreError::RevNotFound {
                rev: rev.to_owned(),
            })?
            .map_err(|e| CoreError::Internal {
                message: format!("revset stream: {e}"),
            })?;

        // Match jj CLI: refuse to silently pick one of several matches (e.g. a divergent change id).
        if block_on(stream.next()).is_some() {
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
