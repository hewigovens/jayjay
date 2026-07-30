use super::*;
use jayjay_core::diff::syntax::SyntaxToken;
use jayjay_core::diff::{ConflictLineKind, ContextRegion, DiffLine, DiffSpan, DiffSpanStyle};

fn line(text: &str) -> DiffLine {
    DiffLine {
        old_line_no: Some(1),
        new_line_no: Some(1),
        style: DiffSpanStyle::Context,
        spans: vec![DiffSpan {
            text: text.to_owned(),
            style: DiffSpanStyle::Context,
            token: SyntaxToken::Plain,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
        context_region: None,
    }
}

fn file_diff() -> Arc<FileDiff> {
    Arc::new(FileDiff {
        path: "a.txt".to_owned(),
        language: "Text".to_owned(),
        lines: vec![line("hello"), line("world")],
        whitespace_only_hidden: false,
    })
}

#[test]
fn unified_reuses_same_allocation_on_hit() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80);
    let second = cache.unified(&fd, 80);
    assert!(
        Arc::ptr_eq(&first, &second),
        "same key should reuse the Arc"
    );
}

#[test]
fn unified_rewraps_when_cols_change() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80);
    let second = cache.unified(&fd, 40);
    assert!(!Arc::ptr_eq(&first, &second), "new cols should rewrap");
}

#[test]
fn unified_rewraps_when_diff_identity_changes() {
    let mut cache = DiffWrapCache::default();
    let fd_a = file_diff();
    let first = cache.unified(&fd_a, 80);
    let fd_b = file_diff();
    let second = cache.unified(&fd_b, 80);
    assert!(
        !Arc::ptr_eq(&first, &second),
        "different diff should rewrap"
    );
}

#[test]
fn unified_rewraps_after_arc_make_mut_changes_the_diff() {
    let mut fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80);

    Arc::make_mut(&mut fd).lines[0].spans[0].text = "changed".to_owned();
    let second = cache.unified(&fd, 80);

    assert!(
        !Arc::ptr_eq(&first, &second),
        "the cache's retained Arc must force make_mut onto a new identity"
    );
    assert_eq!(second[0].line.text(), "changed");
}

#[test]
fn side_by_side_reuses_same_allocation_on_hit() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.side_by_side(&fd, 80, 80);
    let second = cache.side_by_side(&fd, 80, 80);
    assert!(
        Arc::ptr_eq(&first, &second),
        "same key should reuse the Arc"
    );
}

#[test]
fn side_by_side_rewraps_when_a_side_changes() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.side_by_side(&fd, 80, 80);
    let second = cache.side_by_side(&fd, 80, 40);
    assert!(!Arc::ptr_eq(&first, &second), "new cols should rewrap");
}

#[test]
fn side_by_side_context_region_uses_raw_line_mapping_after_conflict_block() {
    let mut conflict_start = line("<<<<<<< conflict");
    conflict_start.conflict_kind = ConflictLineKind::Start;
    let mut conflict_content = line("content");
    conflict_content.conflict_kind = ConflictLineKind::Content;
    let mut conflict_end = line(">>>>>>> conflict");
    conflict_end.conflict_kind = ConflictLineKind::End;
    let region = ContextRegion {
        id: 7,
        old_start_line: 4,
        new_start_line: 4,
        initial_line_count: 20,
        line_count: 20,
    };
    let mut separator = line("20 unchanged lines");
    separator.old_line_no = None;
    separator.new_line_no = None;
    separator.style = DiffSpanStyle::Separator;
    separator.context_region = Some(region);
    let fd = Arc::new(FileDiff {
        path: "conflict.txt".to_owned(),
        language: "Text".to_owned(),
        lines: vec![conflict_start, conflict_content, conflict_end, separator],
        whitespace_only_hidden: false,
    });
    let expected_row = build_side_by_side_rows(&fd.lines)
        .iter()
        .position(|row| row.old.style == DiffSpanStyle::Separator)
        .expect("separator row") as u32;

    let rows = DiffWrapCache::default().side_by_side(&fd, 80, 80);

    assert_eq!(rows[expected_row as usize].row.context_region, Some(region));
}

fn note(id: &str, resolved: bool) -> ReviewNoteStatus {
    use jayjay_review::{NoteEntry, NoteSide};
    ReviewNoteStatus {
        note: NoteEntry {
            id: id.to_owned(),
            change_id: "c1".to_owned(),
            path: "a.txt".to_owned(),
            identity: "id-1".to_owned(),
            side: NoteSide::New,
            line: 1,
            anchor_excerpt: String::new(),
            anchor_context: Vec::new(),
            ignore_whitespace: false,
            body: "note".to_owned(),
            created_at_ms: 0,
            updated_at_ms: 0,
            resolved,
            resolved_at_ms: None,
        },
        status: if resolved {
            jayjay_review::NoteStatus::Resolved
        } else {
            jayjay_review::NoteStatus::Current
        },
        group_index: Some(0),
    }
}

#[test]
fn rows_reuses_same_allocation_when_notes_are_unchanged() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let notes = vec![note("n1", false)];
    let first = cache.rows(&fd, 80, &notes);
    let second = cache.rows(&fd, 80, &notes);
    assert!(
        Arc::ptr_eq(&first, &second),
        "identical notes should reuse the Arc"
    );
}

#[test]
fn rows_rebuilds_when_a_note_changes_status() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let unresolved = vec![note("n1", false)];
    let resolved = vec![note("n1", true)];
    let first = cache.rows(&fd, 80, &unresolved);
    let second = cache.rows(&fd, 80, &resolved);
    assert!(
        !Arc::ptr_eq(&first, &second),
        "a status flip (e.g. an external resolve-note write) must rebuild the row list"
    );
}

#[test]
fn rows_rebuilds_when_cols_change() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let notes = vec![note("n1", false)];
    let first = cache.rows(&fd, 80, &notes);
    let second = cache.rows(&fd, 40, &notes);
    assert!(!Arc::ptr_eq(&first, &second), "new cols should rebuild");
}
