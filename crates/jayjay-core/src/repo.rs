use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use futures::StreamExt as _;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::config::StackedConfig;
use jj_lib::conflicts::{MaterializedTreeValue, materialize_tree_value};
use jj_lib::fileset::FilesetAliasesMap;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::hex_util::encode_reverse_hex;
use jj_lib::local_working_copy::LocalWorkingCopyFactory;
use jj_lib::matchers::EverythingMatcher;
use jj_lib::merged_tree::TreeDiffEntry;
use jj_lib::object_id::ObjectId;
use jj_lib::op_store::RefTarget;
use jj_lib::ref_name::RefName;
use jj_lib::ref_name::WorkspaceNameBuf;
use jj_lib::repo::{ReadonlyRepo, Repo as _, StoreFactories};
use jj_lib::repo_path::RepoPathUiConverter;
use jj_lib::revset::{
    self, RevsetAliasesMap, RevsetDiagnostics, RevsetExtensions, RevsetParseContext,
    RevsetWorkspaceContext, SymbolResolver,
};
use jj_lib::settings::UserSettings;
use jj_lib::time_util::DatePatternContext;
use jj_lib::workspace::{Workspace, WorkingCopyFactories};
use pollster::FutureExt as _;

use crate::types::*;

pub struct Repo {
    path: PathBuf,
    workspace_name: WorkspaceNameBuf,
    repo: RwLock<Arc<ReadonlyRepo>>,
}

fn working_copy_factories() -> WorkingCopyFactories {
    let mut factories: WorkingCopyFactories = HashMap::new();
    factories.insert(
        "local".to_string(),
        Box::new(LocalWorkingCopyFactory {}),
    );
    factories
}

fn default_settings() -> Result<UserSettings, CoreError> {
    let config = StackedConfig::with_defaults();
    UserSettings::from_config(config).map_err(|e| CoreError::Internal {
        message: format!("config error: {e}"),
    })
}

impl Repo {
    pub fn open(path: &Path) -> CoreResult<Self> {
        let settings = default_settings()?;
        let store_factories = StoreFactories::default();
        let wc_factories = working_copy_factories();

        let workspace = Workspace::load(&settings, path, &store_factories, &wc_factories)
            .map_err(|e| CoreError::RepoNotFound {
                path: format!("{}: {e}", path.display()),
            })?;

        let repo = workspace
            .repo_loader()
            .load_at_head()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("failed to load repo: {e}"),
            })?;

        Ok(Self {
            path: workspace.workspace_root().to_owned(),
            workspace_name: workspace.workspace_name().to_owned(),
            repo: RwLock::new(repo),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn path_converter(&self) -> RepoPathUiConverter {
        RepoPathUiConverter::Fs {
            cwd: self.path.clone(),
            base: self.path.clone(),
        }
    }

    fn get_repo(&self) -> Arc<ReadonlyRepo> {
        self.repo.read().unwrap().clone()
    }

    fn set_repo(&self, repo: Arc<ReadonlyRepo>) {
        *self.repo.write().unwrap() = repo;
    }

    fn revset_workspace_context<'a>(
        &'a self,
        path_converter: &'a RepoPathUiConverter,
    ) -> RevsetWorkspaceContext<'a> {
        RevsetWorkspaceContext {
            path_converter,
            workspace_name: self.workspace_name.as_ref(),
        }
    }

    fn resolve_commit(&self, repo: &Arc<ReadonlyRepo>, rev: &str) -> CoreResult<JjCommit> {
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
        let expression = revset::parse(&mut diagnostics, rev, &context).map_err(|e| {
            CoreError::RevNotFound {
                rev: format!("{rev}: {e}"),
            }
        })?;

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

    fn commit_to_change_info(&self, repo: &Arc<ReadonlyRepo>, commit: &JjCommit) -> ChangeInfo {
        let change_id = encode_reverse_hex(commit.change_id().as_bytes());
        let commit_id = commit.id().hex();
        let author = commit.author();
        let bookmarks: Vec<String> = repo
            .view()
            .local_bookmarks_for_commit(commit.id())
            .map(|(name, _)| name.as_str().to_owned())
            .collect();
        let wc_id = repo
            .view()
            .get_wc_commit_id(self.workspace_name.as_ref());
        let is_working_copy = wc_id.is_some_and(|id| id == commit.id());

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
        }
    }

    pub fn log(&self, revset_str: &str) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
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
            revset::parse(&mut diagnostics, revset_str, &context).map_err(|e| {
                CoreError::Internal {
                    message: format!("parse revset: {e}"),
                }
            })?;

        let empty_extensions: &[&Box<dyn revset::SymbolResolverExtension>] = &[];
        let symbol_resolver = SymbolResolver::new(repo.as_ref(), empty_extensions);
        let resolved = expression
            .resolve_user_expression(repo.as_ref(), &symbol_resolver)
            .map_err(|e| CoreError::Internal {
                message: format!("resolve revset: {e}"),
            })?;

        let revset_result = resolved
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::Internal {
                message: format!("eval revset: {e}"),
            })?;

        let mut changes = Vec::new();
        for result in revset_result.iter() {
            let commit_id = result.map_err(|e| CoreError::Internal {
                message: format!("revset iter: {e}"),
            })?;
            let commit =
                repo.store()
                    .get_commit(&commit_id)
                    .map_err(|e| CoreError::Internal {
                        message: format!("get commit: {e}"),
                    })?;
            changes.push(self.commit_to_change_info(&repo, &commit));
        }

        Ok(changes)
    }

    fn materialized_value_to_content(
        &self,
        path: &jj_lib::repo_path::RepoPath,
        value: MaterializedTreeValue,
    ) -> CoreResult<Option<String>> {
        match value {
            MaterializedTreeValue::Absent => Ok(None),
            MaterializedTreeValue::AccessDenied(err) => {
                Ok(Some(format!("<access denied: {err}>")))
            }
            MaterializedTreeValue::File(mut file) => {
                let bytes = file.read_all(path).block_on().map_err(|e| CoreError::Internal {
                    message: format!("read file {}: {e}", path.as_internal_file_string()),
                })?;
                if bytes.contains(&0) {
                    return Ok(Some(format!("<binary file ({} bytes)>", bytes.len())));
                }
                match String::from_utf8(bytes) {
                    Ok(text) => Ok(Some(text)),
                    Err(err) => Ok(Some(format!(
                        "<binary file ({} bytes)>",
                        err.into_bytes().len()
                    ))),
                }
            }
            MaterializedTreeValue::Symlink { target, .. } => Ok(Some(format!("symlink -> {target}"))),
            MaterializedTreeValue::FileConflict(_) => Ok(Some("<conflicted file>".to_owned())),
            MaterializedTreeValue::OtherConflict { .. } => Ok(Some("<conflict>".to_owned())),
            MaterializedTreeValue::GitSubmodule(id) => {
                Ok(Some(format!("<git submodule {}>", id.hex())))
            }
            MaterializedTreeValue::Tree(_) => Ok(Some("<directory>".to_owned())),
        }
    }

    fn diff_hunks_for_commit(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &JjCommit,
    ) -> CoreResult<Vec<DiffHunk>> {
        let before_tree = commit.parent_tree(repo.as_ref()).block_on().map_err(|e| {
            CoreError::Internal {
                message: format!("load parent tree: {e}"),
            }
        })?;
        let after_tree = commit.tree();
        let path_converter = self.path_converter();
        let mut diff_stream = before_tree.diff_stream(&after_tree, &EverythingMatcher);
        let mut diff = Vec::new();

        while let Some(TreeDiffEntry { path, values }) = diff_stream.next().block_on() {
            let values = values.map_err(|e| CoreError::Internal {
                message: format!("tree diff {}: {e}", path.as_internal_file_string()),
            })?;
            let old_value = materialize_tree_value(
                repo.store(),
                &path,
                values.before,
                before_tree.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize old {}: {e}", path.as_internal_file_string()),
            })?;
            let new_value = materialize_tree_value(
                repo.store(),
                &path,
                values.after,
                after_tree.labels(),
            )
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("materialize new {}: {e}", path.as_internal_file_string()),
            })?;

            let hunk_type = match (old_value.is_absent(), new_value.is_absent()) {
                (true, false) => HunkType::Added,
                (false, true) => HunkType::Removed,
                _ => HunkType::Modified,
            };

            diff.push(DiffHunk {
                path: PathBuf::from(path_converter.format_file_path(&path)),
                old_content: self.materialized_value_to_content(&path, old_value)?,
                new_content: self.materialized_value_to_content(&path, new_value)?,
                hunk_type,
            });
        }

        Ok(diff)
    }

    pub fn show(&self, rev: &str) -> CoreResult<ChangeDetail> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let info = self.commit_to_change_info(&repo, &commit);
        let diff = self.diff_hunks_for_commit(&repo, &commit)?;
        Ok(ChangeDetail {
            info,
            diff,
        })
    }

    pub fn describe(&self, rev: &str, message: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut()
            .rewrite_commit(&commit)
            .set_description(message)
            .write()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("describe: {e}"),
            })?;

        tx.repo_mut().rebase_descendants().block_on().map_err(|e| CoreError::Internal {
            message: format!("rebase descendants: {e}"),
        })?;

        let new_repo = tx.commit("describe").block_on().map_err(|e| CoreError::Internal {
            message: format!("commit tx: {e}"),
        })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn new_change(&self, parent_rev: &str, message: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let parent = self.resolve_commit(&repo, parent_rev)?;

        let mut tx = repo.start_transaction();
        let tree = parent.tree();
        let new_commit = tx
            .repo_mut()
            .new_commit(vec![parent.id().clone()], tree)
            .set_description(message)
            .write()
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("new change: {e}"),
            })?;

        // Point working copy to the new commit
        let wc_name = self.workspace_name.clone();
        tx.repo_mut()
            .edit(wc_name, &new_commit)
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("edit working copy: {e}"),
            })?;

        let new_repo = tx.commit("new change").block_on().map_err(|e| CoreError::Internal {
            message: format!("commit tx: {e}"),
        })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn squash(&self, _rev: &str, _into: Option<&str>) -> CoreResult<()> {
        todo!("squash: complex operation, needs CommitWithSelection")
    }

    pub fn abandon(&self, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut().record_abandoned_commit(&commit);
        tx.repo_mut().rebase_descendants().block_on().map_err(|e| CoreError::Internal {
            message: format!("rebase descendants: {e}"),
        })?;

        let new_repo = tx.commit("abandon").block_on().map_err(|e| CoreError::Internal {
            message: format!("commit tx: {e}"),
        })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn rebase(&self, rev: &str, dest: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;
        let dest_commit = self.resolve_commit(&repo, dest)?;

        let mut tx = repo.start_transaction();
        jj_lib::rewrite::rebase_commit(
            tx.repo_mut(),
            commit,
            vec![dest_commit.id().clone()],
        )
        .block_on()
        .map_err(|e| CoreError::Internal {
            message: format!("rebase: {e}"),
        })?;

        tx.repo_mut().rebase_descendants().block_on().map_err(|e| CoreError::Internal {
            message: format!("rebase descendants: {e}"),
        })?;

        let new_repo = tx.commit("rebase").block_on().map_err(|e| CoreError::Internal {
            message: format!("commit tx: {e}"),
        })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn list_bookmarks(&self) -> CoreResult<Vec<BookmarkInfo>> {
        let repo = self.get_repo();
        let mut bookmarks = Vec::new();

        for (name, target) in repo.view().local_bookmarks() {
            if let Some(commit_id) = target.as_normal() {
                let change_id = match repo.store().get_commit(commit_id) {
                    Ok(commit) => encode_reverse_hex(commit.change_id().as_bytes()),
                    Err(_) => String::new(),
                };

                let is_tracking = repo
                    .view()
                    .all_remote_bookmarks()
                    .any(|(sym, _)| sym.name == name);

                bookmarks.push(BookmarkInfo {
                    name: name.as_str().to_owned(),
                    change_id,
                    is_tracking_remote: is_tracking,
                });
            }
        }

        Ok(bookmarks)
    }

    pub fn create_bookmark(&self, name: &str, rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut().set_local_bookmark_target(
            RefName::new(name),
            RefTarget::resolved(Some(commit.id().clone())),
        );

        let new_repo = tx
            .commit("create bookmark")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn move_bookmark(&self, name: &str, to_rev: &str) -> CoreResult<()> {
        let repo = self.get_repo();
        let commit = self.resolve_commit(&repo, to_rev)?;

        let mut tx = repo.start_transaction();
        tx.repo_mut().set_local_bookmark_target(
            RefName::new(name),
            RefTarget::resolved(Some(commit.id().clone())),
        );

        let new_repo = tx
            .commit("move bookmark")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn delete_bookmark(&self, name: &str) -> CoreResult<()> {
        let repo = self.get_repo();

        let mut tx = repo.start_transaction();
        tx.repo_mut().set_local_bookmark_target(
            RefName::new(name),
            RefTarget::absent(),
        );

        let new_repo = tx
            .commit("delete bookmark")
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("commit tx: {e}"),
            })?;
        self.set_repo(new_repo);
        Ok(())
    }

    pub fn git_push(&self, _bookmark: &str) -> CoreResult<()> {
        todo!("git push: needs GitPushRefTargets + subprocess callback")
    }

    pub fn git_fetch(&self, _remote: &str) -> CoreResult<()> {
        todo!("git fetch: needs GitFetch + import_refs")
    }
}
