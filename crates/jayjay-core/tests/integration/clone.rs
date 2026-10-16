use std::fs;
use std::io;
use std::net::TcpListener;
use std::thread;
use std::time::{Duration, Instant};

use jayjay_core::{CloneRequest, JayError, Repo, SyncToken};
use jj_test::LinearFixture;

fn request(source: &str, folder: &std::path::Path, name: &str) -> CloneRequest {
    let location = folder.join(name);
    CloneRequest::new(source, location.to_str().expect("utf-8 path")).expect("valid request")
}

#[test]
fn clones_a_repository_that_opens_at_its_default_bookmark() {
    let source = LinearFixture::build();
    let location = tempfile::tempdir().expect("tempdir");
    let clone = request(
        source.path.to_str().expect("utf-8 path"),
        location.path(),
        "copy",
    );

    clone.run(&SyncToken::standalone()).expect("clone");

    assert!(clone.destination.join("feature.txt").is_file());
    Repo::open(&clone.destination).expect("open clone");
}

#[test]
fn canceling_a_clone_restores_the_destination_it_started_from() {
    for empty_folder_existed in [false, true] {
        // A listener that never answers holds Git in its handshake, so the clone is still running when it is canceled.
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind listener");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let source = format!("git://{}/repo.git", listener.local_addr().expect("address"));
        let location = tempfile::tempdir().expect("tempdir");
        let clone = request(&source, location.path(), "repo");
        if empty_folder_existed {
            fs::create_dir(&clone.destination).expect("create empty destination");
        }
        let sync = SyncToken::standalone();

        let outcome = thread::scope(|scope| {
            let running = scope.spawn(|| clone.run(&sync));
            let deadline = Instant::now() + Duration::from_secs(10);
            let _connection = loop {
                match listener.accept() {
                    Ok((connection, _)) => break connection,
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "git never connected");
                        thread::sleep(Duration::from_millis(20));
                    }
                    Err(error) => panic!("accept: {error}"),
                }
            };
            sync.cancel();
            running.join().expect("clone thread")
        });

        let error = outcome.expect_err("clone should be canceled");
        assert!(matches!(error, JayError::Canceled), "{error}");
        if empty_folder_existed {
            let entries = fs::read_dir(&clone.destination).expect("destination kept");
            assert_eq!(
                entries.count(),
                0,
                "partial clone left in the existing folder"
            );
        } else {
            assert!(!clone.destination.exists(), "partial clone left behind");
        }
    }
}

#[test]
fn rejects_a_destination_that_is_taken() {
    let location = tempfile::tempdir().expect("tempdir");
    fs::create_dir(location.path().join("taken")).expect("create destination");
    fs::write(location.path().join("taken/keep.txt"), "mine\n").expect("write file");
    fs::write(location.path().join("file"), "mine\n").expect("write file");
    let mut taken = vec!["taken", "file"];
    #[cfg(unix)]
    {
        fs::create_dir(location.path().join("empty")).expect("create empty folder");
        std::os::unix::fs::symlink("empty", location.path().join("link")).expect("symlink");
        std::os::unix::fs::symlink("missing", location.path().join("dangling")).expect("symlink");
        taken.extend(["link", "dangling"]);
    }

    for name in taken {
        let clone = request("https://example.invalid/taken.git", location.path(), name);
        let error = clone
            .run(&SyncToken::standalone())
            .expect_err("destination is taken");
        assert!(
            error.to_string().contains("already exists"),
            "{name}: {error}"
        );
    }

    assert!(location.path().join("taken/keep.txt").is_file());
    #[cfg(unix)]
    {
        assert!(location.path().join("link").is_symlink());
        assert!(location.path().join("dangling").is_symlink());
    }
}

#[cfg(unix)]
#[test]
fn a_symlinked_parent_is_resolved_before_the_clone_starts() {
    let root = tempfile::tempdir().expect("tempdir");
    fs::create_dir(root.path().join("real")).expect("create folder");
    std::os::unix::fs::symlink("real", root.path().join("current")).expect("symlink");

    let clone = request(
        "https://example.invalid/repo.git",
        &root.path().join("current"),
        "repo",
    );

    let real = fs::canonicalize(root.path().join("real")).expect("canonical");
    assert_eq!(clone.destination, real.join("repo"));
}
