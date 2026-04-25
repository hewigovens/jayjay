use std::cell::OnceCell;
use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use jj_lib::commit::Commit;
use jj_lib::gitignore::GitIgnoreFile;
use jj_lib::merged_tree::MergedTree;
use jj_lib::ref_name::WorkspaceName;
use jj_lib::repo::{ReadonlyRepo, Repo as _};
use jj_lib::repo_path::RepoPathBuf;

use super::environment;
use super::support::block_on_result;
use crate::types::*;

pub(crate) struct WorkingCopyIgnoreMatcher {
    workspace_root: PathBuf,
    workspace_root_canonical: Option<PathBuf>,
    base_ignores: Arc<GitIgnoreFile>,
    wc_commit: Commit,
    tracked_tree: OnceCell<MergedTree>,
}

impl WorkingCopyIgnoreMatcher {
    pub(crate) fn new(
        repo: &ReadonlyRepo,
        workspace_name: &WorkspaceName,
        workspace_root: &Path,
    ) -> CoreResult<Self> {
        let wc_commit_id = repo
            .view()
            .get_wc_commit_id(workspace_name)
            .ok_or_else(|| CoreError::Internal {
                message: format!(
                    "workspace {} has no working-copy commit",
                    workspace_name.as_symbol()
                ),
            })?;
        let wc_commit = repo
            .store()
            .get_commit(wc_commit_id)
            .map_err(|e| CoreError::Internal {
                message: format!("load working-copy commit: {e}"),
            })?;
        Ok(Self {
            workspace_root: workspace_root.to_path_buf(),
            workspace_root_canonical: workspace_root.canonicalize().ok(),
            base_ignores: base_git_ignores(repo, workspace_root)?,
            wc_commit,
            tracked_tree: OnceCell::new(),
        })
    }

    pub(crate) fn has_unignored_paths(&self, paths: &[String]) -> CoreResult<bool> {
        let mut cache: HashMap<String, Arc<GitIgnoreFile>> = HashMap::new();
        for path in paths {
            if !self.path_is_ignored(Path::new(path), &mut cache)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn path_is_ignored(
        &self,
        event_path: &Path,
        cache: &mut HashMap<String, Arc<GitIgnoreFile>>,
    ) -> CoreResult<bool> {
        let Some(relative) = self.relative_event_path(event_path) else {
            return Ok(false);
        };
        let Some(components) = path_components(&relative) else {
            return Ok(false);
        };
        if components.is_empty() {
            return Ok(false);
        }
        if components
            .iter()
            .any(|name| path_component_is_internal(name))
        {
            return Ok(true);
        }

        let ignores = self.ignores_for_parent_dirs(&components, cache)?;
        let path = components.join("/");
        let matches = if event_path.is_dir() {
            ignores.matches(&format!("{path}/"))
        } else {
            ignores.matches(&path)
        };
        if !matches {
            return Ok(false);
        }
        // Tracked paths still need to surface even when they match ignore rules.
        if self.path_is_tracked(&relative)? {
            return Ok(false);
        }
        Ok(true)
    }

    fn relative_event_path(&self, event_path: &Path) -> Option<PathBuf> {
        if let Ok(relative) = event_path.strip_prefix(&self.workspace_root) {
            return Some(relative.to_path_buf());
        }
        let workspace_root = self.workspace_root_canonical.as_deref()?;
        let event_path = event_path.canonicalize().ok()?;
        event_path
            .strip_prefix(workspace_root)
            .ok()
            .map(Path::to_path_buf)
    }

    fn ignores_for_parent_dirs(
        &self,
        components: &[&str],
        cache: &mut HashMap<String, Arc<GitIgnoreFile>>,
    ) -> CoreResult<Arc<GitIgnoreFile>> {
        let mut prefix = String::new();
        let mut ignores = match cache.get(&prefix) {
            Some(cached) => cached.clone(),
            None => {
                let chain = chain_ignore_file_at(
                    self.base_ignores.clone(),
                    "",
                    self.workspace_root.join(".gitignore"),
                )?;
                cache.insert(prefix.clone(), chain.clone());
                chain
            }
        };
        let mut disk_dir = self.workspace_root.clone();

        for component in components.iter().take(components.len().saturating_sub(1)) {
            let dir_path = format!("{prefix}{component}/");
            if ignores.matches(&dir_path) {
                return Ok(ignores);
            }
            disk_dir.push(component);
            prefix.push_str(component);
            prefix.push('/');
            ignores = match cache.get(&prefix) {
                Some(cached) => cached.clone(),
                None => {
                    let chain =
                        chain_ignore_file_at(ignores, &prefix, disk_dir.join(".gitignore"))?;
                    cache.insert(prefix.clone(), chain.clone());
                    chain
                }
            };
        }

        Ok(ignores)
    }

    fn path_is_tracked(&self, relative: &Path) -> CoreResult<bool> {
        let Ok(repo_path) = RepoPathBuf::from_relative_path(relative) else {
            return Ok(false);
        };
        let tree = self.tracked_tree.get_or_init(|| self.wc_commit.tree());
        let value = block_on_result("load working-copy tree path", tree.path_value(&repo_path))?;
        Ok(value.is_present())
    }
}

pub(crate) fn base_git_ignores(
    repo: &ReadonlyRepo,
    workspace_root: &Path,
) -> CoreResult<Arc<GitIgnoreFile>> {
    let mut ignores = GitIgnoreFile::empty();

    if let Ok(git_backend) = jj_lib::git::get_git_backend(repo.store()) {
        let git_repo = git_backend.git_repo();
        let config = git_repo.config_snapshot();
        if let Some(excludes_file_path) =
            environment::git_excludes_file_path(&config, workspace_root)
        {
            ignores = chain_ignore_file(ignores, excludes_file_path)?;
        }
        let info_exclude = git_backend.git_repo_path().join("info").join("exclude");
        ignores = chain_ignore_file(ignores, info_exclude)?;
    } else if let Ok(git_config) = gix::config::File::from_globals()
        && let Some(excludes_file_path) =
            environment::git_excludes_file_path(&git_config, workspace_root)
    {
        ignores = chain_ignore_file(ignores, excludes_file_path)?;
    }

    Ok(ignores)
}

fn path_components(path: &Path) -> Option<Vec<&str>> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            continue;
        };
        components.push(component.to_str()?);
    }
    Some(components)
}

fn path_component_is_internal(component: &str) -> bool {
    matches!(component, ".jj" | ".git" | ".DS_Store")
}

fn chain_ignore_file(
    ignores: Arc<GitIgnoreFile>,
    path: impl Into<PathBuf>,
) -> CoreResult<Arc<GitIgnoreFile>> {
    chain_ignore_file_at(ignores, "", path)
}

fn chain_ignore_file_at(
    ignores: Arc<GitIgnoreFile>,
    prefix: &str,
    path: impl Into<PathBuf>,
) -> CoreResult<Arc<GitIgnoreFile>> {
    ignores
        .chain_with_file(prefix, path.into())
        .map_err(|e| CoreError::Internal {
            message: format!("process git ignore file: {e}"),
        })
}
