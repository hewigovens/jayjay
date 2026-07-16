use std::fs;
use std::path::PathBuf;

use super::{Store, stored_repository_path};

fn repository(directory: &tempfile::TempDir, name: &str) -> PathBuf {
    let path = directory.path().join(name);
    fs::create_dir_all(&path).unwrap();
    path
}

#[test]
fn pinning_is_ordered_deduplicated_and_persistent() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    let first = repository(&directory, "first");
    let second = repository(&directory, "second");
    let mut store = Store::load_from(path.clone());

    store.set_pinned(&first, true);
    store.set_pinned(&second, true);
    store.set_pinned(&first, true);

    let mut reloaded = Store::load_from(path);
    assert_eq!(
        reloaded.repositories(),
        vec![
            stored_repository_path(&second).unwrap(),
            stored_repository_path(&first).unwrap()
        ]
    );

    reloaded.set_pinned(&first, false);
    assert_eq!(
        reloaded.repositories(),
        vec![stored_repository_path(&second).unwrap()]
    );
}

#[test]
fn mutation_refreshes_external_writes_before_saving() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    let first = repository(&directory, "first");
    let second = repository(&directory, "second");
    let third = repository(&directory, "third");
    let mut long_lived = Store::load_from(path.clone());
    let mut other_shell = Store::load_from(path.clone());

    long_lived.set_pinned(&first, true);
    other_shell.set_pinned(&second, true);
    long_lived.set_pinned(&third, true);

    let mut reloaded = Store::load_from(path);
    assert_eq!(
        reloaded.repositories(),
        vec![
            stored_repository_path(&third).unwrap(),
            stored_repository_path(&second).unwrap(),
            stored_repository_path(&first).unwrap()
        ]
    );
}

#[test]
fn read_detects_an_equal_length_external_replacement() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    let first = r#"{"repositories":["/a"]}"#;
    let second = r#"{"repositories":["/b"]}"#;
    assert_eq!(first.len(), second.len());
    fs::write(&path, first).unwrap();
    let mut store = Store::load_from(path.clone());
    assert_eq!(store.repositories(), vec!["/a"]);

    fs::write(path, second).unwrap();

    assert_eq!(store.repositories(), vec!["/b"]);
}

#[test]
fn loading_removes_empty_and_duplicate_entries() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    fs::write(
        &path,
        r#"{"repositories":["/first","","/first","/second"]}"#,
    )
    .unwrap();
    let mut store = Store::load_from(path);

    assert_eq!(store.repositories(), vec!["/first", "/second"]);
}

#[test]
fn malformed_file_is_preserved_before_reset() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    fs::write(&path, "not json").unwrap();
    let mut store = Store::load_from(path.clone());

    assert!(store.repositories().is_empty());
    assert!(!path.exists());
    assert_eq!(
        fs::read_to_string(path.with_extension("json.corrupt")).unwrap(),
        "not json"
    );
}

#[test]
fn deleting_the_file_clears_a_long_lived_reader() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    let first = repository(&directory, "first");
    let mut store = Store::load_from(path.clone());

    store.set_pinned(&first, true);
    fs::remove_file(path).unwrap();

    assert!(store.repositories().is_empty());
}

#[test]
fn mutation_after_deletion_does_not_restore_old_pins() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("repositories.json");
    let first = repository(&directory, "first");
    let second = repository(&directory, "second");
    let mut store = Store::load_from(path.clone());

    store.set_pinned(&first, true);
    fs::remove_file(&path).unwrap();
    store.set_pinned(&second, true);

    let mut reloaded = Store::load_from(path);
    assert_eq!(
        reloaded.repositories(),
        vec![stored_repository_path(&second).unwrap()]
    );
}

#[test]
fn failed_save_does_not_publish_the_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let store_path = directory.path().join("repositories.json");
    let repository = repository(&directory, "repo");
    fs::create_dir(&store_path).unwrap();
    let mut store = Store::load_from(store_path);

    assert!(store.set_pinned(&repository, true).is_empty());
    assert!(store.repositories().is_empty());
}

#[cfg(unix)]
#[test]
fn non_utf8_path_is_not_persisted_lossily() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let directory = tempfile::tempdir().unwrap();
    let store_path = directory.path().join("repositories.json");
    let invalid = PathBuf::from(OsString::from_vec(vec![b'/', b't', b'm', b'p', b'/', 0xff]));
    let mut store = Store::load_from(store_path.clone());

    assert!(store.set_pinned(&invalid, true).is_empty());
    assert!(!store_path.exists());
}
