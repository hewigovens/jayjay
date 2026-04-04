use std::collections::{HashMap, HashSet};

use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::Repo as _;
use jj_lib::revset::{self, RevsetDiagnostics, RevsetParseContext, SymbolResolver};
use jj_lib::time_util::DatePatternContext;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn log(&self, revset_str: &str) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
        let immutable_ids = self.immutable_commit_ids(&repo);
        let revset_result = self.evaluate_revset(&repo, revset_str)?;

        // First pass: collect changes without divergent info
        let mut changes = Vec::new();
        for result in revset_result.iter() {
            let commit_id = result.map_err(|e| CoreError::Internal {
                message: format!("revset iter: {e}"),
            })?;
            let commit = repo
                .store()
                .get_commit(&commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("get commit: {e}"),
                })?;
            if self.should_include_in_log(&repo, &commit) {
                changes.push(self.commit_to_change_info(
                    &repo,
                    &commit,
                    Some(&immutable_ids),
                    None,
                ));
            }
        }
        // Second pass: mark divergent (change IDs appearing more than once)
        Self::mark_divergent(&mut changes);
        Ok(changes)
    }

    pub fn log_graph(&self, revset_str: &str) -> CoreResult<Vec<GraphEntry>> {
        let repo = self.get_repo();
        let immutable_ids = self.immutable_commit_ids(&repo);
        let revset_result = self.evaluate_revset(&repo, revset_str)?;

        let mut entries = Vec::new();
        for result in revset_result.iter_graph() {
            let (commit_id, edge_list) = result.map_err(|e| CoreError::Internal {
                message: format!("graph iter: {e}"),
            })?;
            let commit = repo
                .store()
                .get_commit(&commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("get commit: {e}"),
                })?;
            if !self.should_include_in_log(&repo, &commit) {
                continue;
            }
            let edges = edge_list
                .into_iter()
                .map(|e| GraphEdge {
                    target: e.target.hex(),
                    edge_type: match e.edge_type {
                        jj_lib::graph::GraphEdgeType::Direct => EdgeType::Direct,
                        jj_lib::graph::GraphEdgeType::Indirect => EdgeType::Indirect,
                        jj_lib::graph::GraphEdgeType::Missing => EdgeType::Missing,
                    },
                })
                .collect();
            entries.push(GraphEntry {
                change: self.commit_to_change_info(&repo, &commit, Some(&immutable_ids), None),
                edges,
            });
        }
        // Mark divergent entries
        let divergent_ids = Self::find_divergent_ids(entries.iter().map(|e| &e.change));
        for entry in &mut entries {
            if divergent_ids.contains(&entry.change.change_id) {
                entry.change.is_divergent = true;
            }
        }
        Ok(entries)
    }

    /// Evaluate `immutable()` once and return the set of commit ID hex strings.
    fn immutable_commit_ids(
        &self,
        repo: &std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
    ) -> HashSet<String> {
        self.evaluate_revset(repo, "immutable()")
            .map(|result| {
                result
                    .iter()
                    .filter_map(|r| r.ok())
                    .map(|id| id.hex())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find change IDs that appear more than once in the given changes.
    fn find_divergent_ids<'a>(changes: impl Iterator<Item = &'a ChangeInfo>) -> HashSet<String> {
        let mut counts: HashMap<&str, u32> = HashMap::new();
        let mut all_ids: Vec<&str> = Vec::new();
        for change in changes {
            *counts.entry(&change.change_id).or_insert(0) += 1;
            all_ids.push(&change.change_id);
        }
        counts
            .into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(id, _)| id.to_owned())
            .collect()
    }

    /// Mark changes with duplicate change IDs as divergent.
    fn mark_divergent(changes: &mut [ChangeInfo]) {
        let divergent = Self::find_divergent_ids(changes.iter());
        for change in changes {
            if divergent.contains(&change.change_id) {
                change.is_divergent = true;
            }
        }
    }

    fn evaluate_revset<'a>(
        &self,
        repo: &'a std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
        revset_str: &str,
    ) -> CoreResult<Box<dyn jj_lib::revset::Revset + 'a>> {
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
        let expression = revset::parse(&mut diagnostics, revset_str, &context).map_err(|e| {
            CoreError::Internal {
                message: format!("parse revset: {e}"),
            }
        })?;

        #[allow(clippy::borrowed_box)]
        let empty_extensions: &[&Box<dyn revset::SymbolResolverExtension>] = &[];
        let symbol_resolver = SymbolResolver::new(repo.as_ref(), empty_extensions);
        let resolved = expression
            .resolve_user_expression(repo.as_ref(), &symbol_resolver)
            .map_err(|e| CoreError::Internal {
                message: format!("resolve revset: {e}"),
            })?;

        resolved
            .evaluate(repo.as_ref())
            .map_err(|e| CoreError::Internal {
                message: format!("eval revset: {e}"),
            })
    }
}
