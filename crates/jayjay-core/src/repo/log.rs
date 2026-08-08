use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use futures::StreamExt as _;
use jj_lib::git::REMOTE_NAME_FOR_LOCAL_GIT_REPO;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo::Repo as _;
use jj_lib::revset::{
    self, RevsetDiagnostics, RevsetParseContext, SymbolResolver, UserRevsetExpression,
};
use jj_lib::time_util::DatePatternContext;
use pollster::FutureExt as _;

use super::Repo;
use crate::types::*;

impl Repo {
    pub fn log(&self, revset_str: &str) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
        let revset = self.evaluate_revset(&repo, revset_str)?;
        self.collect_changes(&repo, revset)
    }

    /// Same as `log`, but takes a pre-built typed revset expression (avoids string formatting).
    pub fn log_typed(&self, expression: Arc<UserRevsetExpression>) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
        let revset = self.evaluate_typed_revset(&repo, expression)?;
        self.collect_changes(&repo, revset)
    }

    fn collect_changes<'a>(
        &self,
        repo: &Arc<ReadonlyRepo>,
        revset: Box<dyn jj_lib::revset::Revset + 'a>,
    ) -> CoreResult<Vec<ChangeInfo>> {
        let immutable_ids = self.immutable_commit_ids(repo);
        let mut changes = Vec::new();
        let mut stream = revset.stream();
        while let Some(result) = stream.next().block_on() {
            let commit_id = result.map_err(|e| CoreError::Internal {
                message: format!("revset stream: {e}"),
            })?;
            let commit = repo
                .store()
                .get_commit(&commit_id)
                .map_err(|e| CoreError::Internal {
                    message: format!("get commit: {e}"),
                })?;
            if self.should_include_in_log(repo, &commit) {
                changes.push(self.commit_to_change_info(repo, &commit, Some(&immutable_ids), None));
            }
        }
        Self::mark_divergent(&mut changes);
        Ok(changes)
    }

    pub fn log_graph(&self, revset_str: &str) -> CoreResult<Vec<GraphEntry>> {
        let repo = self.get_repo();
        let immutable_ids = self.immutable_commit_ids(&repo);
        let revset_result = self.evaluate_revset(&repo, revset_str)?;

        let mut entries = Vec::new();
        let mut stream = revset_result.stream_graph();
        while let Some(result) = stream.next().block_on() {
            let (commit_id, edge_list) = result.map_err(|e| CoreError::Internal {
                message: format!("graph stream: {e}"),
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
            if divergent_ids.contains(&entry.change.change_id.id) {
                entry.change.is_divergent = true;
            }
        }
        Ok(entries)
    }

    /// Refuse to rewrite `commit` (resolved from `rev`) when it is immutable, using the same `immutable()` revset that drives `ChangeInfo::is_immutable`; rewrite paths that bypass the jj CLI get no immutability enforcement from jj-lib and must call this themselves.
    pub(crate) fn ensure_commit_mutable(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &jj_lib::commit::Commit,
        rev: &str,
    ) -> CoreResult<()> {
        if self.is_commit_immutable(repo, commit)? {
            return Err(CoreError::Internal {
                message: format!("{rev} is immutable and cannot be rewritten"),
            });
        }
        Ok(())
    }

    pub(crate) fn is_commit_immutable(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &jj_lib::commit::Commit,
    ) -> CoreResult<bool> {
        let revset = self.evaluate_revset(repo, "immutable()")?;
        revset.containing_fn()(commit.id())
            .block_on()
            .map_err(|e| CoreError::Internal {
                message: format!("immutable check: {e}"),
            })
    }

    /// Evaluate `immutable()` once and return the set of commit ID hex strings.
    fn immutable_commit_ids(
        &self,
        repo: &std::sync::Arc<jj_lib::repo::ReadonlyRepo>,
    ) -> HashSet<String> {
        let Ok(result) = self.evaluate_revset(repo, "immutable()") else {
            return HashSet::new();
        };
        let mut stream = result.stream();
        let mut ids = HashSet::new();
        while let Some(result) = stream.next().block_on() {
            if let Ok(id) = result {
                ids.insert(id.hex());
            }
        }
        ids
    }

    /// Find change IDs that appear more than once in the given changes.
    fn find_divergent_ids<'a>(changes: impl Iterator<Item = &'a ChangeInfo>) -> HashSet<String> {
        let mut counts: HashMap<&str, u32> = HashMap::new();
        let mut all_ids: Vec<&str> = Vec::new();
        for change in changes {
            *counts.entry(&change.change_id.id).or_insert(0) += 1;
            all_ids.push(&change.change_id.id);
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
            if divergent.contains(&change.change_id.id) {
                change.is_divergent = true;
            }
        }
    }

    pub(crate) fn is_change_id_divergent(
        &self,
        repo: &Arc<ReadonlyRepo>,
        change_id: &str,
    ) -> CoreResult<bool> {
        let revset = self.evaluate_revset(repo, &format!("change_id({change_id})"))?;
        let mut count = 0;
        let mut stream = revset.stream();
        while let Some(result) = stream.next().block_on() {
            result.map_err(|e| CoreError::Internal {
                message: format!("revset stream: {e}"),
            })?;
            count += 1;
            if count > 1 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Number of commits matching `expr`. Returns 0 if the revset can't evaluate.
    pub(crate) fn count_revset(&self, repo: &Arc<ReadonlyRepo>, expr: &str) -> u32 {
        let Ok(revset) = self.evaluate_revset(repo, expr) else {
            return 0;
        };
        let mut count = 0u32;
        let mut stream = revset.stream();
        while let Some(result) = stream.next().block_on() {
            if result.is_err() {
                break;
            }
            count = count.saturating_add(1);
        }
        count
    }

    pub(crate) fn evaluate_typed_revset<'a>(
        &self,
        repo: &'a Arc<ReadonlyRepo>,
        expression: Arc<UserRevsetExpression>,
    ) -> CoreResult<Box<dyn jj_lib::revset::Revset + 'a>> {
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

    fn evaluate_revset<'a>(
        &self,
        repo: &'a Arc<ReadonlyRepo>,
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
            extensions: &extensions,
            workspace: Some(self.revset_workspace_context(&path_converter)),
        };

        let mut diagnostics = RevsetDiagnostics::new();
        let expression = revset::parse(&mut diagnostics, revset_str, &context).map_err(|e| {
            CoreError::Internal {
                message: format!("parse revset: {e}"),
            }
        })?;
        self.evaluate_typed_revset(repo, expression)
    }
}
