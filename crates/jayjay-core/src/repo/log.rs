use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use futures::StreamExt as _;
use jj_lib::backend::CommitId;
use jj_lib::config::ConfigGetResultExt as _;
use jj_lib::graph::TopoGroupedGraph;
use jj_lib::object_id::ObjectId;
use jj_lib::repo::ReadonlyRepo;
use jj_lib::repo::Repo as _;
use jj_lib::revset::{self, SymbolResolver, UserRevsetExpression};

use super::Repo;
use super::resolve::{ChangeInfoContext, CommitRefIndex};
use super::revsets::{DEFAULT_LOG_CONTEXT_DEPTH, LogQuery, build_default_revset};
use super::support::{block_on, on_worker_stack};
use crate::dag::DagLayout;
use crate::types::*;

pub(crate) struct ImmutableIds {
    pub(crate) commits: HashSet<String>,
    pub(crate) parents: HashSet<String>,
}

type GraphRowData = (jj_lib::commit::Commit, Vec<GraphEdge>);

/// Collapse jj-lib's per-boundary `Missing` edges into a single one, matching `jj log`.
///
/// For a revset whose selected commits are disconnected from their parents, jj-lib
/// enumerates one missing edge per external boundary edge — hundreds for a deep history.
/// They all mean the same thing ("ancestry continues off-page"), so a node with one parent
/// keeps one termination stub instead of fanning into one lane per boundary commit.
fn collapse_graph_edges(
    edge_list: Vec<jj_lib::graph::GraphEdge<CommitId>>,
    root_commit_id: &CommitId,
) -> Vec<GraphEdge> {
    use jj_lib::graph::GraphEdgeType;
    let mut edges = Vec::with_capacity(edge_list.len());
    let mut missing_target = None;
    for edge in edge_list {
        let edge_type = match edge.edge_type {
            GraphEdgeType::Direct if &edge.target != root_commit_id => EdgeType::Direct,
            GraphEdgeType::Indirect if &edge.target != root_commit_id => EdgeType::Indirect,
            _ => {
                missing_target = Some(edge.target);
                continue;
            }
        };
        edges.push(GraphEdge {
            target: edge.target.hex(),
            edge_type,
        });
    }
    if let Some(target) = missing_target {
        edges.push(GraphEdge {
            target: target.hex(),
            edge_type: EdgeType::Missing,
        });
    }
    edges
}

/// One bounded slice of the log graph: at most `applied_limit` real change rows, its computed layout, and whether the ordered stream held at least one more row beyond that limit.
#[derive(Debug, Clone)]
pub struct LogGraphPage {
    pub entries: Vec<GraphEntry>,
    pub layout: DagLayout,
    pub has_more: bool,
    pub applied_limit: u32,
}

impl Repo {
    pub fn log(&self, revset_str: &str) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
        let revset = self.evaluate_revset(&repo, revset_str)?;
        self.collect_changes(&repo, revset)
    }

    /// Same as `log`, but takes a pre-built typed revset expression (avoids string formatting).
    pub(crate) fn log_typed(
        &self,
        expression: Arc<UserRevsetExpression>,
    ) -> CoreResult<Vec<ChangeInfo>> {
        let repo = self.get_repo();
        let revset = self.evaluate_typed_revset(&repo, expression)?;
        self.collect_changes(&repo, revset)
    }

    fn collect_changes<'a>(
        &self,
        repo: &Arc<ReadonlyRepo>,
        revset: Box<dyn jj_lib::revset::Revset + 'a>,
    ) -> CoreResult<Vec<ChangeInfo>> {
        on_worker_stack(|| {
            let immutable_ids = self.immutable_ids(repo);
            let mut changes = Vec::new();
            let mut stream = revset.stream();
            while let Some(result) = block_on(stream.next()) {
                let commit_id = result.map_err(|e| CoreError::Internal {
                    message: format!("revset stream: {e}"),
                })?;
                let commit =
                    repo.store()
                        .get_commit(&commit_id)
                        .map_err(|e| CoreError::Internal {
                            message: format!("get commit: {e}"),
                        })?;
                if self.should_include_in_log(repo, &commit) {
                    changes.push(self.commit_to_change_info(
                        repo,
                        &commit,
                        ChangeInfoContext {
                            immutable_ids: Some(&immutable_ids),
                            ..ChangeInfoContext::default()
                        },
                    ));
                }
            }
            Self::mark_divergent(&mut changes);
            Ok(changes)
        })
    }

    pub fn log_graph(&self, revset_str: &str) -> CoreResult<Vec<GraphEntry>> {
        let repo = self.get_repo();
        let expression = self.parse_revset_str(&repo, revset_str)?;
        let (rows, _has_more) = self.collect_graph_rows(&repo, &expression, None)?;
        self.materialize_graph_entries(&repo, rows)
    }

    /// Bounded log/graph load: resolves `query` (honouring `revsets.log` in [`LogQuery::Default`]), orders the complete revset through `TopoGroupedGraph` exactly as `log_graph` does, then keeps only the first `limit` rows that pass [`Repo::should_include_in_log`] — metadata and the row layout are never computed for the look-ahead row that decides `has_more`.
    pub fn log_graph_page(&self, query: &LogQuery, limit: u32) -> CoreResult<LogGraphPage> {
        match query {
            LogQuery::Explicit(revset) => self.log_graph_page_for_revset(revset, limit),
            LogQuery::Default => match self.configured_default_log_revset() {
                Some(revset) => self.log_graph_page_for_revset(&revset, limit),
                None => self.log_graph_page_widening_default(limit),
            },
        }
    }

    fn log_graph_page_for_revset(&self, revset_str: &str, limit: u32) -> CoreResult<LogGraphPage> {
        let page_started = Instant::now();
        let span = tracing::debug_span!("log_graph.page", limit);
        let _entered = span.enter();
        let repo = self.get_repo();
        let expression = self.parse_revset_str(&repo, revset_str)?;
        let (rows, has_more) = self.collect_graph_rows(&repo, &expression, Some(limit))?;
        self.log_graph_page_from_rows(&repo, rows, has_more, limit, page_started)
    }

    fn log_graph_page_from_rows(
        &self,
        repo: &Arc<ReadonlyRepo>,
        rows: Vec<GraphRowData>,
        has_more: bool,
        limit: u32,
        page_started: Instant,
    ) -> CoreResult<LogGraphPage> {
        let entries = self.materialize_graph_entries(repo, rows)?;
        let layout_started = Instant::now();
        let layout = {
            let span = tracing::debug_span!("log_graph.layout");
            let _entered = span.enter();
            DagLayout::compute(&entries)
        };
        tracing::debug!(
            elapsed_us = layout_started.elapsed().as_micros() as u64,
            "layout timing"
        );
        tracing::debug!(
            elapsed_us = page_started.elapsed().as_micros() as u64,
            "page timing"
        );
        Ok(LogGraphPage {
            entries,
            layout,
            has_more,
            applied_limit: limit,
        })
    }

    /// [`LogQuery::Default`] with no `revsets.log` override: retries the pinned `builtin_log()` expression at increasing context depths until the page reaches `limit` rows or the immutable-heads context stops growing — each attempt stays bounded by the same post-`TopoGroupedGraph` limit, so a deep, branchy history never renders more than `limit` rows just because a wider depth was needed to reach that many.
    fn log_graph_page_widening_default(&self, limit: u32) -> CoreResult<LogGraphPage> {
        const WIDENING_DEPTHS: [u32; 5] = [DEFAULT_LOG_CONTEXT_DEPTH, 4, 8, 16, 32];
        let page_started = Instant::now();
        let span = tracing::debug_span!("log_graph.page", limit);
        let _entered = span.enter();
        let repo = self.get_repo();
        let mut previous_count = None;
        for &depth in &WIDENING_DEPTHS {
            let expression = self.parse_revset_str(&repo, &build_default_revset(depth))?;
            let (rows, has_more) = self.collect_graph_rows(&repo, &expression, Some(limit))?;
            let reached_target = rows.len() as u32 >= limit;
            let stalled = previous_count == Some(rows.len());
            let done = reached_target || (!has_more && stalled) || depth == WIDENING_DEPTHS[4];
            previous_count = Some(rows.len());
            if done {
                return self.log_graph_page_from_rows(&repo, rows, has_more, limit, page_started);
            }
        }
        unreachable!("WIDENING_DEPTHS is non-empty and its final depth always completes")
    }

    /// Streams `expression` through the same prioritized `TopoGroupedGraph` order `jj log` uses; with `limit`, stops after that many included rows and reports whether the ordered stream held at least one more. Metadata (`commit_to_change_info`, including immutability membership) is computed only for the kept rows, never for a row past the limit — see [`Self::bounded_immutable_ids`].
    fn collect_graph_rows(
        &self,
        repo: &Arc<ReadonlyRepo>,
        expression: &Arc<UserRevsetExpression>,
        limit: Option<u32>,
    ) -> CoreResult<(Vec<GraphRowData>, bool)> {
        on_worker_stack(|| {
            let evaluation_started = Instant::now();
            let (revset_result, prioritized_ids) = {
                let span = tracing::debug_span!("log_graph.revset_evaluation");
                let _entered = span.enter();
                (
                    self.evaluate_typed_revset(repo, expression.clone())?,
                    self.log_graph_prioritized_ids(repo, expression)?,
                )
            };
            tracing::debug!(
                elapsed_us = evaluation_started.elapsed().as_micros() as u64,
                "revset evaluation timing"
            );

            let mut topo_order =
                TopoGroupedGraph::new(revset_result.stream_graph(), |id: &CommitId| id);
            for id in prioritized_ids {
                topo_order.prioritize_branch(id);
            }

            let mut rows = Vec::new();
            let mut has_more = false;
            let root_commit_id = repo.store().root_commit_id();
            let mut stream = std::pin::pin!(topo_order.stream());
            let grouping_started = Instant::now();
            let grouping_span = tracing::debug_span!("log_graph.group_and_limit");
            let grouping_entered = grouping_span.enter();
            while let Some(result) = block_on(stream.next()) {
                let (commit_id, edge_list) = result.map_err(|e| CoreError::Internal {
                    message: format!("graph stream: {e}"),
                })?;
                let commit =
                    repo.store()
                        .get_commit(&commit_id)
                        .map_err(|e| CoreError::Internal {
                            message: format!("get commit: {e}"),
                        })?;
                if !self.should_include_in_log(repo, &commit) {
                    continue;
                }
                if let Some(limit) = limit
                    && rows.len() as u32 >= limit
                {
                    has_more = true;
                    break;
                }
                rows.push((commit, collapse_graph_edges(edge_list, root_commit_id)));
            }
            drop(grouping_entered);
            tracing::debug!(
                elapsed_us = grouping_started.elapsed().as_micros() as u64,
                "grouping timing"
            );

            Ok((rows, has_more))
        })
    }

    fn materialize_graph_entries(
        &self,
        repo: &Arc<ReadonlyRepo>,
        rows: Vec<GraphRowData>,
    ) -> CoreResult<Vec<GraphEntry>> {
        on_worker_stack(|| {
            let metadata_started = Instant::now();
            let metadata_span = tracing::debug_span!("log_graph.metadata", rows = rows.len());
            let metadata_entered = metadata_span.enter();
            let immutability_started = Instant::now();
            let immutable_ids = {
                let span = tracing::debug_span!("log_graph.immutability_membership");
                let _entered = span.enter();
                self.bounded_immutable_ids(repo, rows.iter().map(|(commit, _)| commit))?
            };
            tracing::debug!(
                elapsed_us = immutability_started.elapsed().as_micros() as u64,
                "immutability timing"
            );
            let displayed_commit_ids = rows
                .iter()
                .map(|(commit, _)| commit.id().hex())
                .collect::<HashSet<_>>();
            let ref_index_started = Instant::now();
            let ref_index = {
                let span = tracing::debug_span!("log_graph.ref_index");
                let _entered = span.enter();
                CommitRefIndex::build(repo, self.workspace_name.as_ref(), &displayed_commit_ids)
            };
            tracing::debug!(
                elapsed_us = ref_index_started.elapsed().as_micros() as u64,
                "ref index timing"
            );
            let empty_checks_started = Instant::now();
            let empty_states = {
                let span = tracing::debug_span!("log_graph.empty_checks");
                let _entered = span.enter();
                rows.iter()
                    .map(|(commit, _)| block_on(commit.is_empty(repo.as_ref())).unwrap_or(false))
                    .collect::<Vec<_>>()
            };
            tracing::debug!(
                elapsed_us = empty_checks_started.elapsed().as_micros() as u64,
                "empty checks timing"
            );
            let empty_check_count = empty_states.len();
            let materialization_started = Instant::now();
            let mut entries: Vec<GraphEntry> = {
                let span = tracing::debug_span!("log_graph.commit_materialization");
                let _entered = span.enter();
                rows.into_iter()
                    .zip(empty_states)
                    .map(|((commit, edges), is_empty)| GraphEntry {
                        change: self.commit_to_change_info(
                            repo,
                            &commit,
                            ChangeInfoContext {
                                immutable_ids: Some(&immutable_ids),
                                ref_index: Some(&ref_index),
                                is_empty: Some(is_empty),
                                ..ChangeInfoContext::default()
                            },
                        ),
                        edges,
                    })
                    .collect()
            };
            tracing::debug!(
                elapsed_us = materialization_started.elapsed().as_micros() as u64,
                "commit materialization timing"
            );
            drop(metadata_entered);
            tracing::debug!(
                elapsed_us = metadata_started.elapsed().as_micros() as u64,
                "metadata timing"
            );
            tracing::debug!(
                rows_materialized = entries.len(),
                immutable_ids_enumerated = immutable_ids.commits.len(),
                immutable_parent_ids_enumerated = immutable_ids.parents.len(),
                empty_checks = empty_check_count,
                "log graph work counters"
            );

            // Mark divergent entries
            let divergent_ids = Self::find_divergent_ids(entries.iter().map(|e| &e.change));
            for entry in &mut entries {
                if divergent_ids.contains(&entry.change.change_id.id) {
                    entry.change.is_divergent = true;
                }
            }
            Ok(entries)
        })
    }

    /// The repository's `revsets.log` setting when it actually overrides the pinned default, `None` when unset or still jj's own shipped default (in which case the caller widens the pinned expression itself).
    ///
    /// Reads via the `jj` CLI rather than `repo.settings()`, because `UserSettings` here is built once from user-level config only (see `default_settings()`) and never sees a repository-scoped (`jj config set --repo …`) layer the way the real `jj log` does.
    fn configured_default_log_revset(&self) -> Option<String> {
        // jj's own shipped default for `revsets.log` is the alias name `builtin_log()`, not an expression, and `revset_aliases_map()` can't resolve it — a pre-existing gap: `builtin_log()`'s alias definition is a quoted literal TOML string, which `parse_toml_string()`'s `toml::from_str::<String>` rejects as a document — so treat it exactly like an unset key.
        const PINNED_DEFAULT_ALIAS: &str = "builtin_log()";

        let configured = self
            .run_jj_output(&["config", "get", "revsets.log"])
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_default();
        let trimmed = configured.trim();
        if trimmed.is_empty() || trimmed == PINNED_DEFAULT_ALIAS {
            None
        } else {
            Some(trimmed.to_owned())
        }
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
        self.revset_contains(repo, "immutable()", commit)
    }

    pub(crate) fn has_immutable_child(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commit: &jj_lib::commit::Commit,
    ) -> CoreResult<bool> {
        self.revset_contains(repo, "parents(immutable())", commit)
    }

    fn revset_contains(
        &self,
        repo: &Arc<ReadonlyRepo>,
        revset_str: &str,
        commit: &jj_lib::commit::Commit,
    ) -> CoreResult<bool> {
        let revset = self.evaluate_revset(repo, revset_str)?;
        block_on(revset.containing_fn()(commit.id())).map_err(|e| CoreError::Internal {
            message: format!("{revset_str} check: {e}"),
        })
    }

    fn immutable_ids(&self, repo: &Arc<ReadonlyRepo>) -> ImmutableIds {
        ImmutableIds {
            commits: self.revset_commit_ids(repo, "immutable()"),
            parents: self.revset_commit_ids(repo, "parents(immutable())"),
        }
    }

    /// Immutability membership for exactly `commits`, not the whole repository: intersects `immutable()`
    /// and `parents(immutable())` with an explicit set built from `commits` before evaluating, so the
    /// walk stays bounded by the page size instead of enumerating every immutable commit in the repo
    /// (which can be hundreds of thousands on a large checkout).
    fn bounded_immutable_ids<'a>(
        &self,
        repo: &Arc<ReadonlyRepo>,
        commits: impl Iterator<Item = &'a jj_lib::commit::Commit>,
    ) -> CoreResult<ImmutableIds> {
        let commit_ids: Vec<CommitId> = commits.map(|commit| commit.id().clone()).collect();
        if commit_ids.is_empty() {
            return Ok(ImmutableIds {
                commits: HashSet::new(),
                parents: HashSet::new(),
            });
        }
        let displayed = UserRevsetExpression::commits(commit_ids);
        let immutable = self.parse_revset_str(repo, "immutable()")?;
        let parents_of_immutable = self.parse_revset_str(repo, "parents(immutable())")?;
        Ok(ImmutableIds {
            commits: self.typed_revset_commit_ids(repo, immutable.intersection(&displayed)),
            parents: self
                .typed_revset_commit_ids(repo, parents_of_immutable.intersection(&displayed)),
        })
    }

    /// Evaluate `expression` once and return its commit ID hex strings; an invalid revset yields an empty set so display loading stays resilient.
    fn typed_revset_commit_ids(
        &self,
        repo: &Arc<ReadonlyRepo>,
        expression: Arc<UserRevsetExpression>,
    ) -> HashSet<String> {
        let Ok(result) = self.evaluate_typed_revset(repo, expression) else {
            return HashSet::new();
        };
        let mut stream = result.stream();
        let mut ids = HashSet::new();
        while let Some(result) = block_on(stream.next()) {
            if let Ok(id) = result {
                ids.insert(id.hex());
            }
        }
        ids
    }

    /// Evaluate `revset_str` once and return its commit ID hex strings; an invalid revset yields an empty set so display loading stays resilient.
    fn revset_commit_ids(&self, repo: &Arc<ReadonlyRepo>, revset_str: &str) -> HashSet<String> {
        let Ok(result) = self.evaluate_revset(repo, revset_str) else {
            return HashSet::new();
        };
        on_worker_stack(|| {
            let mut stream = result.stream();
            let mut ids = HashSet::new();
            while let Some(result) = block_on(stream.next()) {
                if let Ok(id) = result {
                    ids.insert(id.hex());
                }
            }
            ids
        })
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
        on_worker_stack(|| {
            let mut count = 0;
            let mut stream = revset.stream();
            while let Some(result) = block_on(stream.next()) {
                result.map_err(|e| CoreError::Internal {
                    message: format!("revset stream: {e}"),
                })?;
                count += 1;
                if count > 1 {
                    return Ok(true);
                }
            }
            Ok(false)
        })
    }

    /// Number of commits matching `expr`. Returns 0 if the revset can't evaluate.
    pub(crate) fn count_revset(&self, repo: &Arc<ReadonlyRepo>, expr: &str) -> u32 {
        let Ok(revset) = self.evaluate_revset(repo, expr) else {
            return 0;
        };
        on_worker_stack(|| {
            let mut count = 0u32;
            let mut stream = revset.stream();
            while let Some(result) = block_on(stream.next()) {
                if result.is_err() {
                    break;
                }
                count = count.saturating_add(1);
            }
            count
        })
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
        let expression = self.parse_revset_str(repo, revset_str)?;
        self.evaluate_typed_revset(repo, expression)
    }

    fn parse_revset_str(
        &self,
        repo: &Arc<ReadonlyRepo>,
        revset_str: &str,
    ) -> CoreResult<Arc<UserRevsetExpression>> {
        let settings = repo.settings();
        let aliases_map = self.revset_aliases_map(settings)?;
        let fileset_aliases_map = self.fileset_aliases_map(settings)?;
        self.parse_revset(
            &aliases_map,
            &fileset_aliases_map,
            settings.user_email(),
            revset_str,
        )
        .map_err(|e| CoreError::Internal {
            message: format!("parse revset: {e}"),
        })
    }

    /// Commit IDs matching `revsets.log-graph-prioritize`, intersected with `expression`, in the order the config revset yields them. Empty when the config key is unset.
    fn log_graph_prioritized_ids(
        &self,
        repo: &Arc<ReadonlyRepo>,
        expression: &Arc<UserRevsetExpression>,
    ) -> CoreResult<Vec<CommitId>> {
        let prioritize_revset_str = repo
            .settings()
            .get_string(["revsets", "log-graph-prioritize"])
            .optional()
            .map_err(|e| CoreError::Internal {
                message: format!("read revsets.log-graph-prioritize: {e}"),
            })?
            .unwrap_or_default();
        if prioritize_revset_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let prioritize_expression = self.parse_revset_str(repo, &prioritize_revset_str)?;
        let intersected = prioritize_expression.intersection(expression);
        let revset = self.evaluate_typed_revset(repo, intersected)?;

        let mut ids = Vec::new();
        let mut stream = revset.stream();
        while let Some(result) = block_on(stream.next()) {
            ids.push(result.map_err(|e| CoreError::Internal {
                message: format!("revsets.log-graph-prioritize stream: {e}"),
            })?);
        }
        Ok(ids)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jj_lib::graph::GraphEdge as JjEdge;

    const ROOT: &str = "0000000000000000000000000000000000000000";

    fn cid(hex: &'static str) -> CommitId {
        CommitId::from_hex(hex)
    }

    #[test]
    fn many_missing_edges_collapse_to_one_keeping_direct_and_indirect() {
        let root = cid(ROOT);
        let a = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let b = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        let c = "cccccccccccccccccccccccccccccccccccccccc";
        let d = "dddddddddddddddddddddddddddddddddddddddd";
        let edges = vec![
            JjEdge::indirect(cid(a)),
            JjEdge::missing(cid(b)),
            JjEdge::missing(cid(c)),
            JjEdge::direct(cid(d)),
        ];

        let collapsed = collapse_graph_edges(edges, &root);

        assert_eq!(
            collapsed,
            vec![
                GraphEdge {
                    target: a.to_owned(),
                    edge_type: EdgeType::Indirect
                },
                GraphEdge {
                    target: d.to_owned(),
                    edge_type: EdgeType::Direct
                },
                GraphEdge {
                    target: c.to_owned(),
                    edge_type: EdgeType::Missing
                },
            ]
        );
    }

    #[test]
    fn a_single_parent_off_page_node_keeps_one_termination() {
        let root = cid(ROOT);
        let edges = (0u8..50)
            .map(|i| JjEdge::missing(CommitId::new(vec![i; 20])))
            .collect();

        let collapsed = collapse_graph_edges(edges, &root);

        assert_eq!(collapsed.len(), 1);
        assert_eq!(collapsed[0].edge_type, EdgeType::Missing);
    }

    #[test]
    fn root_targeted_edges_are_treated_as_missing() {
        let root = cid(ROOT);
        let collapsed = collapse_graph_edges(vec![JjEdge::direct(root.clone())], &root);

        assert_eq!(collapsed.len(), 1);
        assert_eq!(collapsed[0].edge_type, EdgeType::Missing);
    }
}
