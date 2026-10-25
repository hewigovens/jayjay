use super::{
    BisectModel, DiffCache, GraphData, LoadingState, NotesState, RepoViewModel, ShownDiff,
    StatsState,
};
use crate::app::{config, recent_revsets};
use crate::diff::{DetailMode, DiffViewMode};
use gpui::{AppContext, Context, SharedString};
use jayjay_core::dag::{DagLayout, OrderedSelection};
use jayjay_core::{
    BookmarkInfo, ChangeInfo, GraphEntry, Repo, RevsetFilterState, RevsetPreset, RevsetVocabulary,
    TagInfo, WorkspaceInfo, default_revset_depth, default_revset_preset,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

struct OpenedRepo {
    repo: Arc<Repo>,
    repo_root_path: String,
    default_revset: RevsetPreset,
    entries: Vec<GraphEntry>,
    bookmarks: Vec<BookmarkInfo>,
    tags: Vec<TagInfo>,
    vocabulary: RevsetVocabulary,
    workspaces: Vec<WorkspaceInfo>,
    pr_host_name: Option<String>,
    fix_unavailable_reason: Option<String>,
}

impl RepoViewModel {
    pub fn new(path: PathBuf) -> Self {
        let repo_path: SharedString = path.display().to_string().into();
        match Self::open_blocking(path) {
            Ok(loaded) => Self::ready(repo_path, loaded),
            Err(e) => Self::error(repo_path, format!("{e}")),
        }
    }

    pub fn opening(path: PathBuf) -> Self {
        Self::empty(path.display().to_string().into())
    }

    /// Keeps window-open off the UI thread, since open/revset eval is slow on large checkouts.
    pub fn open_async(&mut self, cx: &mut Context<Self>) {
        let path = PathBuf::from(self.repo_path.as_ref());
        self.begin_refreshing(cx);
        Self::background_update(
            cx,
            async move {
                let started = std::time::Instant::now();
                let opened = Self::open_blocking(path);
                match &opened {
                    Ok(loaded) => log::info!(
                        "repo opened in {}ms with {} changes",
                        started.elapsed().as_millis(),
                        loaded.entries.len()
                    ),
                    Err(error) => log::warn!("repo open failed: {error}"),
                }
                opened
            },
            move |vm, opened, cx| {
                vm.finish_repo_task(cx);
                match opened {
                    Ok(loaded) => {
                        *vm = Self::ready(vm.repo_path.clone(), loaded);
                        vm.finish_open(cx);
                    }
                    Err(e) => vm.present_error(e),
                }
                cx.notify();
            },
        );
    }

    pub(crate) async fn open_detached(
        path: PathBuf,
        cx: &mut gpui::AsyncApp,
    ) -> jayjay_core::JayResult<gpui::Entity<Self>> {
        let repo_path: SharedString = path.display().to_string().into();
        let loaded = cx
            .background_spawn(async move { Self::open_blocking(path) })
            .await?;
        Ok(cx.new(|_| Self::ready(repo_path, loaded)))
    }

    pub(crate) fn finish_open(&mut self, cx: &mut Context<Self>) {
        config::update(cx, |config| {
            config.record_opened_repo(Path::new(self.repo_path.as_ref()));
        });
        let root = Path::new(self.repo_root_path.as_ref());
        self.revset_filter.recent = recent_revsets::update(cx, |store| store.recent(root));
        self.boot(cx);
    }

    fn open_blocking(path: PathBuf) -> jayjay_core::JayResult<OpenedRepo> {
        let repo_root_path = jayjay_core::workspace_primary_root(&path.to_string_lossy())
            .unwrap_or_else(|| path.to_string_lossy().into_owned());
        let repo = Repo::open(&path)?;
        let default_revset = repo.default_revset_preset();
        let entries = repo.log_graph(&default_revset.revset)?;
        let bookmarks = repo.list_bookmarks().unwrap_or_default();
        let tags = repo.list_tags().unwrap_or_default();
        let vocabulary = repo.revset_vocabulary(&bookmarks);
        let workspaces = repo.workspace_list().unwrap_or_default();
        let pr_host_name = repo.pr_host_name();
        let fix_unavailable_reason = repo.fix_unavailable_reason();
        Ok(OpenedRepo {
            repo: Arc::new(repo),
            repo_root_path,
            default_revset,
            entries,
            bookmarks,
            tags,
            vocabulary,
            workspaces,
            pr_host_name,
            fix_unavailable_reason,
        })
    }

    fn ready(repo_path: SharedString, loaded: OpenedRepo) -> Self {
        let OpenedRepo {
            repo,
            repo_root_path,
            default_revset,
            entries,
            bookmarks,
            tags,
            vocabulary,
            workspaces,
            pr_host_name,
            fix_unavailable_reason,
        } = loaded;
        let revset_filter = RevsetFilterState::new(&default_revset.revset);
        let selected = entries
            .iter()
            .position(|e| e.change.is_working_copy)
            .or(if entries.is_empty() { None } else { Some(0) });
        let dag_layout = Arc::new(DagLayout::compute(&entries));
        let changes: Vec<ChangeInfo> = entries.iter().map(|e| e.change.clone()).collect();
        let mut selected_changes = OrderedSelection::default();
        if let Some(change) = selected.and_then(|selected| changes.get(selected)) {
            selected_changes.replace(change.selection_revision().to_owned());
        }
        Self {
            repo: Some(repo),
            fix_unavailable_reason: fix_unavailable_reason.map(SharedString::from),
            repo_path,
            repo_root_path: repo_root_path.into(),
            error: None,
            working_copy_stale: false,
            selected,
            selected_changes,
            files: None,
            tracked_lfs_paths: Arc::default(),
            conflicted_paths: Arc::default(),
            selected_file_ix: None,
            shown: ShownDiff::default(),
            diff_cache: DiffCache::default(),
            stats: StatsState::default(),
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            language: None,
            can_load_more: default_revset_depth(&revset_filter.revset)
                .is_some_and(|depth| changes.len() >= depth as usize),
            revset_filter,
            default_revset,
            vocabulary,
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name: pr_host_name.map(SharedString::from),
            compare: None,
            graph: GraphData {
                changes: Arc::new(changes),
                entries: Arc::new(entries),
                dag_layout,
                bookmarks: Arc::new(bookmarks),
                tags: Arc::new(tags),
                workspaces: Arc::new(workspaces),
            },
            loading: LoadingState::default(),
            last_internal_mutation_at: None,
            refresh_suspended: false,
            notes: NotesState::default(),
            pending_file_selection: None,
            pending_error: None,
            bisect: BisectModel::default(),
            selection_cache: RefCell::new(None),
        }
    }

    fn empty(repo_path: SharedString) -> Self {
        Self {
            repo: None,
            fix_unavailable_reason: None,
            repo_root_path: repo_path.clone(),
            repo_path,
            error: None,
            working_copy_stale: false,
            selected: None,
            selected_changes: OrderedSelection::default(),
            files: None,
            tracked_lfs_paths: Arc::default(),
            conflicted_paths: Arc::default(),
            selected_file_ix: None,
            shown: ShownDiff::default(),
            diff_cache: DiffCache::default(),
            stats: StatsState::default(),
            working_copy_stats: None,
            current_operation_description: String::new(),
            view_mode: DiffViewMode::Unified,
            ignore_whitespace: false,
            language: None,
            revset_filter: RevsetFilterState::new(&default_revset_preset().revset),
            default_revset: default_revset_preset(),
            can_load_more: false,
            vocabulary: RevsetVocabulary::default(),
            detail_mode: DetailMode::Diff,
            annotate_lines: None,
            avatar_in_flight: HashSet::new(),
            pr_info: None,
            pr_host_name: None,
            compare: None,
            graph: GraphData::default(),
            loading: LoadingState::default(),
            last_internal_mutation_at: None,
            refresh_suspended: false,
            notes: NotesState::default(),
            pending_file_selection: None,
            pending_error: None,
            bisect: BisectModel::default(),
            selection_cache: RefCell::new(None),
        }
    }

    fn error(repo_path: SharedString, msg: String) -> Self {
        let mut vm = Self::empty(repo_path);
        vm.present_error(msg);
        vm
    }

    pub fn boot(&mut self, cx: &mut Context<Self>) {
        self.sync_diff_settings(cx);
        cx.observe_global::<config::AppConfigStore>(|vm, cx| vm.sync_diff_settings(cx))
            .detach();
        cx.on_app_quit(|vm, _| {
            vm.stop_bisect_for_shutdown();
            if let Some(repo) = &vm.repo {
                repo.cancel_running_jj_processes();
            }
            async {}
        })
        .detach();
        // Snapshot small repos on open so the WC is current; huge checkouts defer (snapshot is slow).
        if self
            .repo
            .as_ref()
            .is_some_and(|repo| !repo.working_copy_is_large())
        {
            self.refresh(false, cx);
        } else if let Some(ix) = self.selected {
            self.select_change(ix, cx);
        }
    }
}
