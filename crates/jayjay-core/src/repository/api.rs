use super::session::{Backend, Repo};
use crate::overview::OverviewSnapshot;
use crate::*;
use serde_json::Value;

macro_rules! arg_type {
    (Text) => { &str }; (OptionalText) => { Option<&str> }; (Strings) => { &[String] };
    (Location) => { &str }; (OptionalLocation) => { Option<&str> };
    (FileEditor) => { &FileEditorData }; (ConflictEditor) => { &ConflictEditorData };
    (Selections) => { &[DiffEditFileSelection] };
    (Layers) => { Vec<SubmitStackLayer> }; (Bookmarks) => { &[BookmarkInfo] }; (Sync) => { &SyncToken };
    ($t:ident) => { $t };
}
macro_rules! owned_type {
    (Text) => { String }; (OptionalText) => { Option<String> }; (Strings) => { Vec<String> };
    (Location) => { String }; (OptionalLocation) => { Option<String> };
    (FileEditor) => { FileEditorData }; (ConflictEditor) => { ConflictEditorData };
    (Selections) => { Vec<DiffEditFileSelection> };
    (Layers) => { Vec<SubmitStackLayer> }; (Bookmarks) => { Vec<BookmarkInfo> }; (Sync) => { () };
    ($t:ident) => { $t };
}
macro_rules! borrow_arg {
    (Text, $v:ident, $s:ident) => {
        $v.as_str()
    };
    (Location, $v:ident, $s:ident) => {
        $v.as_str()
    };
    (OptionalText, $v:ident, $s:ident) => {
        $v.as_deref()
    };
    (OptionalLocation, $v:ident, $s:ident) => {
        $v.as_deref()
    };
    (Strings, $v:ident, $s:ident) => {
        $v.as_slice()
    };
    (Selections, $v:ident, $s:ident) => {
        $v.as_slice()
    };
    (Bookmarks, $v:ident, $s:ident) => {
        $v.as_slice()
    };
    (FileEditor, $v:ident, $s:ident) => {
        &$v
    };
    (ConflictEditor, $v:ident, $s:ident) => {
        &$v
    };
    (Sync, $v:ident, $s:ident) => {{
        let () = $v;
        $s
    }};
    ($t:ident, $v:ident, $s:ident) => {
        $v
    };
}
macro_rules! wire_arg {
    (Sync, $v:ident, $r:ident) => {
        Value::Null
    };
    (Location, $v:ident, $r:ident) => {
        Value::String($r.unqualify($v)?)
    };
    (OptionalLocation, $v:ident, $r:ident) => {
        $v.map(|v| $r.unqualify(v)).transpose()?.into()
    };
    ($t:ident, $v:ident, $r:ident) => {
        serde_json::to_value($v).map_err(JayError::internal)?
    };
}
macro_rules! host_arg {
    (Location, $v:ident) => {
        crate::ssh::host_path(&$v).to_string_lossy().into_owned()
    };
    (OptionalLocation, $v:ident) => {
        $v.map(|v| crate::ssh::host_path(&v).to_string_lossy().into_owned())
    };
    ($t:ident, $v:ident) => {
        $v
    };
}
macro_rules! qualifies {
    () => {
        false
    };
    (paths) => {
        true
    };
}
macro_rules! sync_arg {
    (Sync, $v:ident) => {
        Some($v)
    };
    ($t:ident, $v:ident) => {
        None::<&SyncToken>
    };
}

fn or_default<T: Default>(read: impl FnOnce() -> JayResult<T>) -> T {
    read().unwrap_or_default()
}

macro_rules! api {
    (fallible { $($(@$mark:ident)? fn $name:ident($($arg:ident: $kind:ident),*) -> $ret:ty;)* }
     values { $(fn $value:ident($($varg:ident: $vkind:ident),*) -> $vret:ty;)* }) => {
        impl Repo {
            $(pub fn $name(&self, $($arg: arg_type!($kind)),*) -> JayResult<$ret> {
                match &self.backend {
                    Backend::Local(repo) => repo.$name($($arg),*),
                    Backend::Remote(repo) => repo.call(stringify!($name), vec![$(wire_arg!($kind, $arg, repo)),*], { let tokens: &[Option<&SyncToken>] = &[$(sync_arg!($kind, $arg)),*]; tokens.iter().copied().flatten().next() }, qualifies!($($mark)?)),
                }
            })*
            $(pub fn $value(&self, $($varg: arg_type!($vkind)),*) -> $vret {
                match &self.backend {
                    Backend::Local(repo) => repo.$value($($varg),*),
                    Backend::Remote(repo) => or_default(|| repo.call(stringify!($value), vec![$(wire_arg!($vkind, $varg, repo)),*], None, false)),
                }
            })*
        }
        pub(crate) fn dispatch(repo: &crate::repo::Repo, method: &str, args: Vec<Value>, sync_token: &SyncToken) -> JayResult<Value> {
            let mut args = args.into_iter();
            match method {
                $(stringify!($name) => {
                    $(let $arg: owned_type!($kind) = serde_json::from_value(args.next().ok_or_else(|| JayError::refused("Missing SSH argument."))?).map_err(JayError::internal)?;
                    let $arg = host_arg!($kind, $arg);)*
                    if args.next().is_some() { return Err(JayError::refused("Unexpected SSH argument.")); }
                    let mut value = serde_json::to_value(repo.$name($(borrow_arg!($kind, $arg, sync_token)),*)?).map_err(JayError::internal)?;
                    if qualifies!($($mark)?) { crate::ssh::wire_paths(&mut value); }
                    Ok(value)
                },)*
                $(stringify!($value) => {
                    $(let $varg: owned_type!($vkind) = serde_json::from_value(args.next().ok_or_else(|| JayError::refused("Missing SSH argument."))?).map_err(JayError::internal)?;)*
                    if args.next().is_some() { return Err(JayError::refused("Unexpected SSH argument.")); }
                    serde_json::to_value(repo.$value($(borrow_arg!($vkind, $varg, sync_token)),*)).map_err(JayError::internal)
                },)*
                _ => Err(JayError::refused(format!("Unsupported SSH repository operation: {method}."))),
            }
        }
    }
}

api! {
    fallible {
        @paths fn repository_context() -> RepositoryContext;
        fn run_jj_command(command: Text) -> JjCommandResult;
        fn refresh_working_copy() -> ();
        fn snapshot_operation() -> String;
        fn has_unignored_working_copy_paths(paths: Strings) -> bool;
        fn annotate_file(rev: Text, path: Text) -> Vec<AnnotationLine>;
        fn file_history(path: Text) -> Vec<ChangeInfo>;
        fn describe(rev: Text, message: Text) -> ();
        fn new_change(parent_rev: Text, message: Text) -> ();
        fn new_change_inserted(rev: Text, position: InsertPosition, message: Text) -> ();
        fn squash(rev: Text, into: OptionalText) -> ();
        fn edit(rev: Text) -> ();
        fn abandon(rev: Text) -> ();
        fn abandon_many(revs: Strings) -> ();
        fn rebase(rev: Text, dest: Text, mode: RebaseMode) -> String;
        fn rebase_many(revs: Strings, dest: Text) -> ();
        fn squash_many(revs: Strings) -> String;
        fn merge(parent_revs: Strings) -> ();
        fn duplicate(rev: Text) -> ();
        fn absorb(rev: Text) -> MutationEffect;
        fn revert_change(rev: Text) -> ();
        fn split(rev: Text, paths: Strings, message: Text, parallel: bool) -> ();
        fn evolog(rev: Text) -> Vec<EvologEntry>;
        fn log(revset_str: Text) -> Vec<ChangeInfo>;
        fn check_revset(revset_str: Text) -> ();
        fn log_graph(revset_str: Text) -> Vec<GraphEntry>;
        fn list_tags() -> Vec<TagInfo>;
        fn create_tag(name: Text, rev: Text) -> ();
        fn delete_tag(name: Text) -> ();
        fn delete_tag_and_push(name: Text, sync: Sync) -> String;
        fn working_copy_file_editor(path: Text) -> FileEditorData;
        fn apply_working_copy_file_editor(data: FileEditor, content: Text) -> ();
        fn restore_files(rev: Text, from: OptionalText, paths: Strings) -> ();
        fn restore_version(rev: Text, version: Text) -> ();
        fn delete_files(paths: Strings) -> ();
        fn ignore_and_untrack(paths: Strings) -> ();
        fn move_to_working_copy(rev: Text, paths: Strings) -> ();
        fn list_bookmarks() -> Vec<BookmarkInfo>;
        fn create_bookmark(name: Text, rev: Text) -> ();
        fn move_bookmark(name: Text, to_rev: Text) -> ();
        fn delete_bookmark(name: Text) -> ();
        fn remove_bookmark_from_rev(name: Text, rev: Text) -> ();
        fn forget_bookmark(name: Text) -> ();
        fn rename_bookmark(old_name: Text, new_name: Text) -> ();
        fn track_bookmark(name: Text, remote: Text) -> ();
        fn forget_stale_bookmarks() -> u32;
        fn show_summary(rev: Text) -> ChangeDetail;
        fn show(rev: Text) -> ChangeDetail;
        fn file_content(rev: Text, path: Text) -> String;
        fn show_file(rev: Text, path: Text) -> DiffHunk;
        fn show_file_rename(rev: Text, old_path: Text, new_path: Text) -> DiffHunk;
        fn show_file_raw(rev: Text, path: Text) -> DiffHunk;
        fn show_file_rename_raw(rev: Text, old_path: Text, new_path: Text) -> DiffHunk;
        fn diff_stats(rev: Text) -> DiffStats;
        fn diff_file_stats(rev: Text, ignore_whitespace: bool) -> Vec<FileDiffStats>;
        fn interdiff_summary(from_rev: Text, to_rev: Text) -> ChangeDetail;
        fn interdiff_file(from_rev: Text, to_rev: Text, path: Text) -> DiffHunk;
        fn interdiff_file_raw(from_rev: Text, to_rev: Text, path: Text) -> DiffHunk;
        fn interdiff_file_with_mode(from_rev: Text, to_rev: Text, path: Text, projection_mode: DiffProjectionMode) -> DiffHunk;
        fn interdiff_file_rename_with_mode(from_rev: Text, to_rev: Text, old_path: Text, new_path: Text, projection_mode: DiffProjectionMode) -> DiffHunk;
        fn resolve_list(rev: Text) -> Vec<String>;
        fn resolve_with_tool(rev: Text, path: Text, tool: Text) -> ();
        fn resolve_with_merge_editor(rev: Text, path: Text) -> ();
        fn conflict_editor(rev: Text, path: Text) -> ConflictEditorData;
        fn apply_conflict_editor(rev: Text, data: ConflictEditor, content: Text) -> ();
        fn resolve_use_ours(rev: Text, path: Text) -> ();
        fn resolve_use_theirs(rev: Text, path: Text) -> ();
        fn op_log() -> Vec<OpLogEntry>;
        fn op_restore(op_id: Text) -> ();
        fn is_at_operation_head() -> bool;
        fn parallelize(revs: Strings) -> MutationEffect;
        @paths fn overview_snapshot() -> OverviewSnapshot;
        @paths fn overview() -> Overview;
        fn submit_stack(layers: Layers) -> StackedPrResult;
        fn detect_stack(base_rev: Text, tip_rev: Text) -> Stack;
        fn bisect_run_step(command: Text, sync: Sync) -> Option<BisectState>;
        fn bisect_run_to_end(command: Text, sync: Sync) -> Option<BisectState>;
        fn start_bisect(range: Text) -> BisectState;
        fn start_bisect_between(first: Text, second: Text) -> BisectState;
        fn bisect_stop() -> ();
        fn shutdown_bisect() -> ();
        fn bisect_restart() -> BisectState;
        fn bisect_dismiss() -> ();
        fn bisect_mark(commit_id: Text, verdict: BisectVerdict) -> BisectState;
        @paths fn workspace_list() -> Vec<WorkspaceInfo>;
        fn workspace_add(dest: Location, name: Text, rev: Text) -> String;
        fn workspace_forget(name: Text, expected_root: OptionalLocation) -> ();
        fn update_stale_workspace() -> ();
        fn workspace_forget_and_delete(name: Text, expected_root: Location) -> Option<String>;
        fn pull_request_open_url(bookmark: Text) -> String;
        @paths fn pull_request_import_preview(url: Text, sync: Sync) -> PullRequestImportPreview;
        @paths fn pull_request_import(url: Text, previewed_head_commit_id: Text, workspace_name: Text, workspace_dest: Location, sync: Sync) -> String;
        fn diff_excerpt() -> Option<DiffExcerpt>;
        fn fix(revs: Strings) -> FixSummary;
        fn jj_commit(message: Text) -> ();
        fn git_lfs_paths(paths: Strings) -> Vec<String>;
        fn submodule_statuses() -> Vec<GitSubmoduleStatus>;
        fn commit_safe_submodule_updates(message: Text, paths: Strings) -> String;
        fn git_push(bookmark: Text, sync: Sync) -> String;
        fn git_push_tag(tag: Text, sync: Sync) -> String;
        fn git_fetch(remote: Text, sync: Sync) -> FetchResult;
        fn git_pull_bookmark(bookmark: Text, sync: Sync) -> FetchResult;
        fn apply_diff_selection(rev: Text, destination: DiffEditDestination, selections: Selections, message: Text, ignore_whitespace: bool) -> ();
    }
    values {
        fn working_copy_is_large() -> bool;
        fn merge_editor_name() -> Option<String>;
        fn current_operation_description() -> String;
        fn trunk_name() -> String;
        fn default_revset_preset() -> RevsetPreset;
        fn revset_vocabulary(bookmarks: Bookmarks) -> RevsetVocabulary;
        fn bisect_state() -> Option<BisectState>;
        fn workspace_presence() -> WorkspacePresence;
        fn pull_request_info(bookmark: Text) -> Option<PrInfo>;
        fn pr_host_name() -> Option<String>;
        fn fix_unavailable_reason() -> Option<String>;
        fn remote_web_url() -> Option<String>;
    }
}
