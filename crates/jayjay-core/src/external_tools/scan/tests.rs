use std::fs;

use crate::HunkType;

use super::{JJ_INSTRUCTIONS, scan_external_diff};

#[test]
fn scans_changed_added_removed_and_nested_files() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("same.txt"), "same\n").expect("same left");
    fs::write(right.path().join("same.txt"), "same\n").expect("same right");
    fs::write(left.path().join("changed.txt"), "old\n").expect("changed left");
    fs::write(right.path().join("changed.txt"), "new\n").expect("changed right");
    fs::write(left.path().join("removed.txt"), "gone\n").expect("removed");
    fs::create_dir(right.path().join("nested")).expect("nested");
    fs::write(right.path().join("nested/added.txt"), "added\n").expect("added");
    fs::write(right.path().join(JJ_INSTRUCTIONS), "ignore me").expect("instructions");

    let entries = scan_external_diff(left.path(), right.path(), true).expect("diff");
    let summary: Vec<(&str, HunkType)> = entries
        .iter()
        .map(|entry| (entry.hunk.path.as_str(), entry.hunk.hunk_type))
        .collect();
    assert_eq!(
        summary,
        vec![
            ("changed.txt", HunkType::Modified),
            ("nested/added.txt", HunkType::Added),
            ("removed.txt", HunkType::Removed),
        ]
    );
    assert_eq!(entries[0].hunk.old.content.as_deref(), Some("old\n"));
    assert_eq!(entries[0].hunk.new.content.as_deref(), Some("new\n"));
}

#[test]
fn read_only_diff_keeps_a_tracked_instructions_file() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join(JJ_INSTRUCTIONS), "old\n").expect("left instructions");
    fs::write(right.path().join(JJ_INSTRUCTIONS), "new\n").expect("right instructions");

    let entries = scan_external_diff(left.path(), right.path(), false).expect("diff");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].hunk.path, JJ_INSTRUCTIONS);
    assert_eq!(entries[0].hunk.old.content.as_deref(), Some("old\n"));
    assert_eq!(entries[0].hunk.new.content.as_deref(), Some("new\n"));
}

#[test]
fn binary_files_get_bounded_placeholders() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("data.bin"), [0, 1]).expect("left binary");
    fs::write(right.path().join("data.bin"), [0, 2]).expect("right binary");

    let entries = scan_external_diff(left.path(), right.path(), false).expect("diff");
    assert_eq!(
        entries[0].hunk.old.content.as_deref(),
        Some("<binary file (2 bytes)>")
    );
    assert_eq!(
        entries[0].hunk.new.content.as_deref(),
        Some("<binary file (2 bytes)>")
    );
}

#[test]
fn pairs_single_files_even_when_temporary_names_differ() {
    let root = tempfile::tempdir().expect("root");
    let left = root.path().join("pre-image.tmp");
    let right = root.path().join("src.rs");
    fs::write(&left, "old\n").expect("left");
    fs::write(&right, "new\n").expect("right");

    let entries = scan_external_diff(&left, &right, false).expect("diff");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].hunk.path, "src.rs");
    assert_eq!(entries[0].hunk.hunk_type, HunkType::Modified);
}

#[cfg(unix)]
#[test]
fn scans_executable_bit_only_changes() {
    use std::os::unix::fs::PermissionsExt as _;

    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    let left_file = left.path().join("script.sh");
    let right_file = right.path().join("script.sh");
    fs::write(&left_file, "#!/bin/sh\n").expect("left");
    fs::write(&right_file, "#!/bin/sh\n").expect("right");
    fs::set_permissions(&left_file, fs::Permissions::from_mode(0o644)).expect("left mode");
    fs::set_permissions(&right_file, fs::Permissions::from_mode(0o755)).expect("right mode");

    let entries = scan_external_diff(left.path(), right.path(), false).expect("diff");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].hunk.path, "script.sh");
    assert_eq!(entries[0].old_executable, Some(false));
    assert_eq!(entries[0].new_executable, Some(true));
}
