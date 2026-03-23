use std::collections::HashMap;
use std::sync::Arc;

use jj_lib::commit::Commit as JjCommit;
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    self, RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetParseContext,
    RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::time_util::DatePatternContext;

use super::Repo;
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
        let aliases_map = RevsetAliasesMap::default();
        let fileset_aliases_map = FilesetAliasesMap::default();
        let extensions = RevsetExtensions::default();
        let path_converter = self.path_converter();

        let context = RevsetParseContext {
            aliases_map: &aliases_map,
            local_variables: HashMap::new(),
            user_email: settings.user_email(),
            date_pattern_context: DatePatternContext::from(chrono::Local::now()),
            default_ignored_remote: Some(REMOTE_NAME_FOR_LOCAL_GIT_REPO),
            fileset_aliases_map: &fileset_aliases_map,
            use_glob_by_default: true,
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

        let commit_id = revset
            .iter()
            .next()
            .ok_or_else(|| CoreError::RevNotFound {
                rev: rev.to_owned(),
            })?
            .map_err(|e| CoreError::Internal {
                message: format!("revset iter: {e}"),
            })?;

        repo.store()
            .get_commit(&commit_id)
            .map_err(|e| CoreError::Internal {
                message: format!("get commit: {e}"),
            })
    }

    pub(crate) fn commit_to_change_info(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let author = commit.author();
        let bookmarks: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        let wc_id = repo.view().get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = wc_id.is_some_and(|id| id == commit.id());
        let has_conflict = commit.has_conflict();
        let is_empty = commit.is_empty(repo.as_ref()).unwrap_or(false);

        ChangeInfo {
            change_id,
            commit_id,
            description: commit.description().to_owned(),
            author: author.name.clone(),
            email: author.email.clone(),
            timestamp_millis: author.timestamp.timestamp.0,
            parents: commit.parent_ids().iter().map(|id| id.hex()).collect(),
            bookmarks,
            is_working_copy,
            has_conflict,
            is_empty,
        }
    }

    pub(crate) fn should_include_in_log(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> bool {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let description = commit.description().trim();
        let bookmarks: Vec<_> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .collect();
        let wc_id = repo.view().get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = wc_id.is_some_and(|id| id == commit.id());

        if !is_working_copy && description.is_empty() && bookmarks.is_empty() {
            let all_zero_commit = commit_id.chars().all(|c| c == '0');
            let all_z_change = change_id.chars().all(|c| c == 'z');
            let no_parents = commit.parent_ids().is_empty();
            if all_zero_commit || all_z_change || no_parents {
                return false;
            }
        }

        true
    }
}
