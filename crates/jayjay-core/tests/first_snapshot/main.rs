//! Its own test binary, so the process-wide revset alias cache is still cold when the snapshot runs; the integration binary warms it through other tests first.

use std::fs;
use std::time::Duration;

use jayjay_core::Repo;
use jj_test::init_jj_repo;

#[test]
fn the_first_snapshot_of_a_process_does_not_deadlock_on_the_write_lock() {
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(120));
        eprintln!("snapshot deadlocked");
        std::process::exit(1);
    });
    let temp_dir = init_jj_repo();
    let repo_path = temp_dir.path().join("repo");
    let repo = Repo::open(&repo_path).expect("open repo");
    fs::write(repo_path.join("hello.txt"), "first edit\n").expect("edit");

    repo.refresh_working_copy().expect("snapshot");

    assert!(!repo.log("@").expect("log")[0].is_empty);
}
