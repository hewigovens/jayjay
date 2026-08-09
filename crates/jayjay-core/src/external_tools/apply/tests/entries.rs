use super::*;

#[test]
fn empty_file_existence_can_be_selected_independently_of_lines() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(right.path().join("added.txt"), "").expect("added empty file");
    fs::write(left.path().join("removed.txt"), "").expect("removed empty file");
    let selections = discarded_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("discard");

    assert!(!right.path().join("added.txt").exists());
    assert!(right.path().join("removed.txt").is_file());
    assert_eq!(
        fs::read(right.path().join("removed.txt")).expect("restored empty file"),
        Vec::<u8>::new()
    );
}

#[test]
fn whole_file_selection_restores_binary_bytes() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("data.bin"), [0, 1, 2]).expect("left binary");
    fs::write(right.path().join("data.bin"), [0, 3, 4]).expect("right binary");
    let selections = discarded_loaded_selections(left.path(), right.path());

    assert_eq!(selections[0].whole_file_side, Some(ExternalDiffSide::Old));
    apply_external_diff_selections(left.path(), right.path(), &selections, false)
        .expect("restore binary");

    assert_eq!(
        fs::read(right.path().join("data.bin")).expect("restored binary"),
        [0, 1, 2]
    );
}

#[cfg(unix)]
#[test]
fn whole_file_selection_restores_a_symlink() {
    use std::os::unix::fs::symlink;

    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    symlink("old-target", left.path().join("link")).expect("left symlink");
    symlink("new-target", right.path().join("link")).expect("right symlink");
    let selections = discarded_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false)
        .expect("restore symlink");

    assert_eq!(
        fs::read_link(right.path().join("link")).expect("restored symlink"),
        PathBuf::from("old-target")
    );
}

#[cfg(unix)]
#[test]
fn restoring_a_deleted_file_restores_its_executable_bit() {
    use std::os::unix::fs::PermissionsExt as _;

    let right = tempfile::tempdir().expect("right");
    let selection = ExternalDiffSelection {
        file: DiffEditFileSelection {
            path: "script.sh".to_owned(),
            old_path: None,
            old_content: Some("#!/bin/sh\n".to_owned()),
            new_content: None,
            hunk_type: HunkType::Removed,
            line_ranges: vec![],
        },
        selected_exists: true,
        selected_executable: Some(true),
        whole_file_side: None,
    };

    apply_external_diff_selections(right.path(), right.path(), &[selection], false)
        .expect("restore");

    let restored = right.path().join("script.sh");
    assert_eq!(
        fs::read_to_string(&restored).expect("content"),
        "#!/bin/sh\n"
    );
    assert_ne!(
        fs::metadata(restored)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o111,
        0
    );
}
