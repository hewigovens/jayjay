use std::cell::Cell;
use std::path::PathBuf;

use notify::event::{CreateKind, MetadataKind, ModifyKind};
use notify::{Event, EventKind};

use super::*;

fn classifier() -> PathClassifier {
    PathClassifier {
        op_heads_dir: PathBuf::from("/repo/.jj/repo/op_heads/heads"),
        repo_root: PathBuf::from("/repo"),
    }
}

fn event(kind: EventKind, path: &str) -> Event {
    Event::new(kind).add_path(PathBuf::from(path))
}

#[test]
fn metadata_only_events_are_ignored() {
    let c = classifier();
    let e = event(
        EventKind::Modify(ModifyKind::Metadata(MetadataKind::Any)),
        "/repo/src/main.rs",
    );
    assert_eq!(c.classify(&e), EventClass::Ignore);
}

#[test]
fn op_heads_writes_classify_as_op() {
    let c = classifier();
    let e = event(
        EventKind::Create(CreateKind::File),
        "/repo/.jj/repo/op_heads/heads/abc123",
    );
    assert_eq!(c.classify(&e), EventClass::OpHeads);
}

#[test]
fn other_jj_internals_are_ignored() {
    let c = classifier();
    let e = event(
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
        "/repo/.jj/working_copy/checkout",
    );
    assert_eq!(c.classify(&e), EventClass::Ignore);
}

#[test]
fn working_copy_edits_classify_as_working_copy() {
    let c = classifier();
    let e = event(
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
        "/repo/src/main.rs",
    );
    assert_eq!(c.classify(&e), EventClass::WorkingCopy);
}

#[test]
fn op_heads_events_are_debounced() {
    let c = classifier();
    let d = Mutex::new(Debounce::fresh());
    let e = event(
        EventKind::Create(CreateKind::File),
        "/repo/.jj/repo/op_heads/heads/abc",
    );
    let relevant = |_: &[PathBuf]| true;

    let t0 = Instant::now();
    assert_eq!(
        next_event(&c, &d, &e, t0, &relevant),
        Some(FsEvent::OpHeads)
    );
    // A second event inside the window is suppressed.
    assert_eq!(
        next_event(&c, &d, &e, t0 + OP_DEBOUNCE / 2, &relevant),
        None
    );
    // Once the window elapses, it emits again.
    assert_eq!(
        next_event(&c, &d, &e, t0 + OP_DEBOUNCE, &relevant),
        Some(FsEvent::OpHeads)
    );
}

#[test]
fn working_copy_events_are_debounced() {
    let c = classifier();
    let d = Mutex::new(Debounce::fresh());
    let e = event(
        EventKind::Modify(ModifyKind::Data(notify::event::DataChange::Content)),
        "/repo/src/main.rs",
    );
    let relevant = |_: &[PathBuf]| true;

    let t0 = Instant::now();
    assert_eq!(
        next_event(&c, &d, &e, t0, &relevant),
        Some(FsEvent::WorkingCopy)
    );
    assert_eq!(
        next_event(&c, &d, &e, t0 + WC_DEBOUNCE / 2, &relevant),
        None
    );
    assert_eq!(
        next_event(&c, &d, &e, t0 + WC_DEBOUNCE, &relevant),
        Some(FsEvent::WorkingCopy)
    );
}

#[test]
fn relevance_filter_is_skipped_during_debounce_window() {
    let c = classifier();
    let d = Mutex::new(Debounce::fresh());
    let e = event(
        EventKind::Create(CreateKind::File),
        "/repo/target/debug/build.o",
    );
    let calls = Cell::new(0u32);
    let relevant = |_: &[PathBuf]| {
        calls.set(calls.get() + 1);
        true
    };

    let t0 = Instant::now();
    // First event runs the filter once and emits.
    assert_eq!(
        next_event(&c, &d, &e, t0, &relevant),
        Some(FsEvent::WorkingCopy)
    );
    assert_eq!(calls.get(), 1, "first event runs the filter");

    // A build storm: many events inside the window must not touch the expensive matcher.
    for i in 1..200 {
        let _ = next_event(
            &c,
            &d,
            &e,
            t0 + WC_DEBOUNCE / 4 + Duration::from_micros(i),
            &relevant,
        );
    }
    assert_eq!(
        calls.get(),
        1,
        "events inside the debounce window must not invoke the gitignore matcher"
    );
}

#[test]
fn irrelevant_working_copy_paths_do_not_emit_or_stamp() {
    let c = classifier();
    let d = Mutex::new(Debounce::fresh());
    let e = event(
        EventKind::Create(CreateKind::File),
        "/repo/target/debug/x.o",
    );
    let ignored = |_: &[PathBuf]| false;
    let tracked = |_: &[PathBuf]| true;

    let t0 = Instant::now();
    // A gitignored path passes the window but the filter rejects it: no emit, no stamp.
    assert_eq!(next_event(&c, &d, &e, t0, &ignored), None);
    // Because the window was never stamped, a relevant edit right after still emits.
    assert_eq!(
        next_event(&c, &d, &e, t0 + Duration::from_millis(1), &tracked),
        Some(FsEvent::WorkingCopy)
    );
}

#[test]
fn cli_operations_reach_primary_and_secondary_workspace_watchers() {
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    let fixture = jj_test::LinearFixture::build();
    let parent = fixture.path.parent().expect("fixture parent").to_path_buf();
    let secondary = parent.join("watched-secondary");
    jj_test::run_jj_in(
        &fixture.path,
        &[
            "workspace",
            "add",
            "--name",
            "watched-secondary",
            secondary.to_str().expect("utf-8 workspace path"),
        ],
    );

    for (label, watched) in [("primary", fixture.path.clone()), ("secondary", secondary)] {
        let (tx, rx) = flume::unbounded::<FsEvent>();
        let filter: IsRelevantWcChange = Arc::new(|_| true);
        let _watcher = RepoFsWatcher::new(&watched, tx, filter).expect("start watcher");
        std::thread::sleep(Duration::from_millis(500));
        while rx.try_recv().is_ok() {}

        let added = parent.join(format!("added-from-{label}"));
        jj_test::run_jj_in(
            &fixture.path,
            &[
                "workspace",
                "add",
                "--name",
                &format!("added-from-{label}"),
                added.to_str().expect("utf-8"),
            ],
        );

        let deadline = Instant::now() + Duration::from_secs(10);
        let mut seen = Vec::new();
        while Instant::now() < deadline && !seen.contains(&FsEvent::OpHeads) {
            if let Ok(event) = rx.recv_timeout(Duration::from_millis(100)) {
                seen.push(event);
            }
        }
        assert!(
            seen.contains(&FsEvent::OpHeads),
            "{label}: a CLI operation must reach the watcher as OpHeads, saw {seen:?}"
        );
    }
}
