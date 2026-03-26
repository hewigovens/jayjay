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
    self, FunctionCallNode, LoweringContext, RemoteRefSymbolExpression, RevsetAliasesMap,
    RevsetDiagnostics, RevsetExtensions, RevsetParseContext, RevsetParseError,
    RevsetWorkspaceContext, SymbolResolver, UserRevsetExpression,
};
use jj_lib::settings::UserSettings;
use jj_lib::str_util::StringExpression;
use jj_lib::time_util::DatePatternContext;

use super::Repo;
use crate::types::*;

impl Repo {
    pub(crate) fn revset_extensions(&self) -> RevsetExtensions {
        let mut extensions = RevsetExtensions::new();
        extensions.add_custom_function("trunk", Self::trunk_revset_function);
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

    fn trunk_expression() -> Arc<UserRevsetExpression> {
        let candidates = [
            Self::remote_bookmark_expression("main", "origin"),
            Self::remote_bookmark_expression("master", "origin"),
            Self::remote_bookmark_expression("trunk", "origin"),
            Self::remote_bookmark_expression("main", "upstream"),
            Self::remote_bookmark_expression("master", "upstream"),
            Self::remote_bookmark_expression("trunk", "upstream"),
            jj_lib::revset::RevsetExpression::root(),
        ];
        jj_lib::revset::RevsetExpression::union_all(&candidates).latest(1)
    }

    fn remote_bookmark_expression(name: &str, remote: &str) -> Arc<UserRevsetExpression> {
        jj_lib::revset::RevsetExpression::remote_bookmarks(
            RemoteRefSymbolExpression {
                name: StringExpression::exact(name),
                remote: StringExpression::exact(remote),
            },
            None,
        )
    }

    pub(crate) fn revset_aliases_map(
        &self,
        settings: &UserSettings,
    ) -> CoreResult<RevsetAliasesMap> {
        let mut aliases_map = RevsetAliasesMap::new();
        for name in settings.table_keys("revset-aliases") {
            let definition = settings
                .get_string(["revset-aliases", name])
                .map_err(|e| CoreError::Internal {
                    message: format!("load revset alias {name}: {e}"),
                })?;
            aliases_map
                .insert(name, definition)
                .map_err(|e| CoreError::Internal {
                    message: format!("parse revset alias {name}: {e}"),
                })?;
        }
        Ok(aliases_map)
    }

    pub(crate) fn fileset_aliases_map(
        &self,
        settings: &UserSettings,
    ) -> CoreResult<FilesetAliasesMap> {
        let mut aliases_map = FilesetAliasesMap::new();
        for name in settings.table_keys("fileset-aliases") {
            let definition = settings
                .get_string(["fileset-aliases", name])
                .map_err(|e| CoreError::Internal {
                    message: format!("load fileset alias {name}: {e}"),
                })?;
            aliases_map
                .insert(name, definition)
                .map_err(|e| CoreError::Internal {
                    message: format!("parse fileset alias {name}: {e}"),
                })?;
        }
        Ok(aliases_map)
    }

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
