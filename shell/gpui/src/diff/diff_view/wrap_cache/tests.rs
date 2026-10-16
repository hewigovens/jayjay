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
            tooltip: None,
        }],
        conflict_kind: ConflictLineKind::None,
        no_eof_newline: false,
        context_region: None,
        moved: None,
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
fn review_rows_reuse_mappings_until_the_diff_changes() {
    let mut removed = line("old");
    removed.style = DiffSpanStyle::Removed;
    removed.new_line_no = None;
    let mut added = line("new");
    added.style = DiffSpanStyle::Added;
    added.old_line_no = None;
    let mut fd = file_diff();
    Arc::make_mut(&mut fd).lines = vec![removed, added.clone(), line("context"), added];
    let mut cache = DiffWrapCache::default();
    let first = cache.review_rows(&fd);
    assert_eq!(first.group_count, 2);
    assert_eq!(
        first.unified_line_groups,
        vec![Some(0), Some(0), None, Some(1)]
    );
    assert_eq!(first.side_by_side_row_groups, vec![Some(0), None, Some(1)]);

    cache.unified(&fd, 80, 0);
    cache.unified(&fd, 40, 0);
    cache.side_by_side(&fd, 40, 40, 0);
    assert!(Arc::ptr_eq(&first, &cache.review_rows(&fd)));

    Arc::make_mut(&mut fd)
        .lines
        .insert(0, line("expanded context"));
    let expanded = cache.review_rows(&fd);
    assert!(!Arc::ptr_eq(&first, &expanded));
    assert_eq!(
        expanded.unified_line_groups,
        vec![None, Some(0), Some(0), None, Some(1)]
    );
    assert_eq!(
        expanded.side_by_side_row_groups,
        vec![None, Some(0), None, Some(1)]
    );
    assert!(Arc::ptr_eq(&expanded, &cache.review_rows(&fd)));
}

#[test]
fn unified_reuses_same_allocation_on_hit() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80, 0);
    let second = cache.unified(&fd, 80, 0);
    assert!(
        Arc::ptr_eq(&first, &second),
        "same key should reuse the Arc"
    );
}

#[test]
fn unified_rewraps_when_cols_change() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80, 0);
    let second = cache.unified(&fd, 40, 0);
    assert!(!Arc::ptr_eq(&first, &second), "new cols should rewrap");
}

#[test]
fn unified_rewraps_when_diff_identity_changes() {
    let mut cache = DiffWrapCache::default();
    let fd_a = file_diff();
    let first = cache.unified(&fd_a, 80, 0);
    let fd_b = file_diff();
    let second = cache.unified(&fd_b, 80, 0);
    assert!(
        !Arc::ptr_eq(&first, &second),
        "different diff should rewrap"
    );
}

#[test]
fn unified_rewraps_after_arc_make_mut_changes_the_diff() {
    let mut fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.unified(&fd, 80, 0);

    Arc::make_mut(&mut fd).lines[0].spans[0].text = "changed".to_owned();
    let second = cache.unified(&fd, 80, 0);

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
    let first = cache.side_by_side(&fd, 80, 80, 0);
    let second = cache.side_by_side(&fd, 80, 80, 0);
    assert!(
        Arc::ptr_eq(&first, &second),
        "same key should reuse the Arc"
    );
}

#[test]
fn side_by_side_rewraps_when_a_side_changes() {
    let fd = file_diff();
    let mut cache = DiffWrapCache::default();
    let first = cache.side_by_side(&fd, 80, 80, 0);
    let second = cache.side_by_side(&fd, 80, 40, 0);
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

    let rows = DiffWrapCache::default().side_by_side(&fd, 80, 80, 0);

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
    let first = cache.rows(&fd, 80, &notes, 0);
    let second = cache.rows(&fd, 80, &notes, 0);
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
    let first = cache.rows(&fd, 80, &unresolved, 0);
    let second = cache.rows(&fd, 80, &resolved, 0);
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
    let first = cache.rows(&fd, 80, &notes, 0);
    let second = cache.rows(&fd, 40, &notes, 0);
    assert!(!Arc::ptr_eq(&first, &second), "new cols should rebuild");
}

#[test]
fn pinned_source_line_survives_reflow_in_both_layouts() {
    use super::super::row_anchor::VisibleRows;

    let long = "wrapped ".repeat(12);
    let mut fd = file_diff();
    Arc::make_mut(&mut fd).lines = vec![line(&long), line("short"), line(&long), line(&long)];
    let mut cache = DiffWrapCache::default();
    let unified = |cache: &mut DiffWrapCache, cols| VisibleRows::Unified {
        diff: fd.clone(),
        lines: cache.unified(&fd, cols, 0),
        rendered: cache.rows(&fd, cols, &[], 0),
    };
    let first_row_of = |cache: &mut DiffWrapCache, cols, source| {
        cache
            .unified(&fd, cols, 0)
            .iter()
            .position(|line| line.line_ix == source)
            .unwrap()
    };

    let wide = unified(&mut cache, 40);
    assert_eq!(cache.show(wide), None);
    let pinned = first_row_of(&mut cache, 40, 3) + 1;
    cache.pin_row(pinned as f32);
    let narrow = unified(&mut cache, 20);
    let expected = first_row_of(&mut cache, 20, 3) + 1;
    assert_eq!(
        cache.show(narrow).map(|row| row as usize),
        Some(expected),
        "a third into the line stays a third into it"
    );

    let sbs = |cache: &mut DiffWrapCache, cols| VisibleRows::SideBySide {
        diff: fd.clone(),
        rows: cache.side_by_side(&fd, cols, cols, 0),
    };
    let wide = sbs(&mut cache, 40);
    cache.show(wide);
    let row_ix = cache
        .side_by_side(&fd, 40, 40, 0)
        .iter()
        .position(|row| row.row_ix == 2)
        .unwrap();
    cache.pin_row(row_ix as f32 + 0.5);
    let narrow = sbs(&mut cache, 20);
    let start = cache
        .side_by_side(&fd, 20, 20, 0)
        .iter()
        .position(|row| row.row_ix == 2)
        .unwrap();
    assert_eq!(cache.show(narrow), Some(start as f32 + 0.5 * 5. / 3.));
}

#[test]
fn binary_size_tooltips_reach_both_cached_layouts() {
    let fd = Arc::new(jayjay_core::diff::compute_file_diff(
        "blob.bin",
        "<binary file (141080 bytes)>",
        "<binary file (141081 bytes)>",
        false,
    ));
    let mut cache = DiffWrapCache::default();
    let unified = cache.unified(&fd, 80, 0);
    assert_eq!(unified.len(), 2);
    assert_eq!(unified[0].line.text(), unified[1].line.text());
    assert!(unified[0].line.text().contains("KB"));
    let old_tooltip = unified[0].line.spans[0]
        .tooltip
        .as_ref()
        .expect("old size tooltip");
    let new_tooltip = unified[1].line.spans[0]
        .tooltip
        .as_ref()
        .expect("new size tooltip");
    assert_ne!(old_tooltip, new_tooltip);
    let sbs = cache.side_by_side(&fd, 80, 80, 0);
    assert_eq!(sbs.len(), 1);
    assert_eq!(sbs[0].row.old.spans[0].tooltip.as_ref(), Some(old_tooltip));
    assert_eq!(sbs[0].row.new.spans[0].tooltip.as_ref(), Some(new_tooltip));
    assert_eq!(fd.lines[0].text(), "<binary file (141080 bytes)>");
}

#[test]
fn label_width_changes_reflow_both_layouts_and_their_render_rows() {
    let mut moved = line(&"a".repeat(25));
    moved.style = DiffSpanStyle::Removed;
    moved.new_line_no = None;
    moved.moved = Some(jayjay_core::diff::MovedRow {
        counterpart_line: 9,
        starts_block: true,
    });
    let mut fd = file_diff();
    Arc::make_mut(&mut fd).lines = vec![moved];
    let mut cache = DiffWrapCache::default();
    assert_eq!(cache.unified(&fd, 30, 0).len(), 1);
    assert_eq!(cache.rows(&fd, 30, &[], 0).rows.len(), 1);
    assert_eq!(cache.side_by_side(&fd, 30, 30, 0).len(), 1);
    assert_eq!(cache.unified(&fd, 30, 14).len(), 2);
    assert_eq!(cache.rows(&fd, 30, &[], 14).rows.len(), 2);
    assert_eq!(cache.side_by_side(&fd, 30, 30, 14).len(), 2);
}
