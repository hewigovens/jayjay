use super::*;

#[test]
fn keeps_a_selected_file_to_directory_transition() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("item"), "old file\n").expect("left file");
    fs::create_dir(right.path().join("item")).expect("right directory");
    fs::write(right.path().join("item/new.txt"), "new file\n").expect("right file");
    let selections = all_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("apply");

    assert!(right.path().join("item").is_dir());
    assert_eq!(
        fs::read_to_string(right.path().join("item/new.txt")).expect("right file"),
        "new file\n"
    );
}

#[test]
fn keeps_a_selected_directory_to_file_transition() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::create_dir(left.path().join("item")).expect("left directory");
    fs::write(left.path().join("item/old.txt"), "old file\n").expect("left file");
    fs::write(right.path().join("item"), "new file\n").expect("right file");
    let selections = all_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("apply");

    assert!(right.path().join("item").is_file());
    assert_eq!(
        fs::read_to_string(right.path().join("item")).expect("right file"),
        "new file\n"
    );
}

#[test]
fn discards_a_file_to_directory_transition() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("item"), "old file\n").expect("left file");
    fs::create_dir(right.path().join("item")).expect("right directory");
    fs::write(right.path().join("item/new.txt"), "new file\n").expect("right file");
    let selections = discarded_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("discard");

    assert!(right.path().join("item").is_file());
    assert_eq!(
        fs::read_to_string(right.path().join("item")).expect("restored file"),
        "old file\n"
    );
}

#[test]
fn rejects_a_partial_topology_transition_before_mutating_output() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::write(left.path().join("item"), "old file\n").expect("left file");
    fs::create_dir(right.path().join("item")).expect("right directory");
    fs::write(right.path().join("item/new.txt"), "new file\n").expect("right file");
    let mut selections = all_loaded_selections(left.path(), right.path());
    let parent = selections
        .iter_mut()
        .find(|selection| selection.file.path == "item")
        .expect("parent selection");
    parent.file.line_ranges.clear();
    parent.selected_exists = true;
    parent.selected_executable = Some(false);

    let error = apply_external_diff_selections(left.path(), right.path(), &selections, false)
        .expect_err("contradictory topology");

    assert!(error.to_string().contains("cannot keep both"));
    assert!(right.path().join("item").is_dir());
    assert_eq!(
        fs::read_to_string(right.path().join("item/new.txt")).expect("right file"),
        "new file\n"
    );
}

#[test]
fn discards_a_directory_to_file_transition() {
    let left = tempfile::tempdir().expect("left");
    let right = tempfile::tempdir().expect("right");
    fs::create_dir(left.path().join("item")).expect("left directory");
    fs::write(left.path().join("item/old.txt"), "old file\n").expect("left file");
    fs::write(right.path().join("item"), "new file\n").expect("right file");
    let selections = discarded_loaded_selections(left.path(), right.path());

    apply_external_diff_selections(left.path(), right.path(), &selections, false).expect("discard");

    assert!(right.path().join("item").is_dir());
    assert_eq!(
        fs::read_to_string(right.path().join("item/old.txt")).expect("restored file"),
        "old file\n"
    );
}
