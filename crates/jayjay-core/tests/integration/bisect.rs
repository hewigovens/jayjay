use std::fs;

use jayjay_core::{
    BisectPhase, BisectResultKind, BisectState, BisectVerdict, JayError, Repo, compose_bisect_range,
};
use jj_test::{BisectStack, changed_paths, find_change, run_jj_in};

fn started() -> (BisectStack, Repo) {
    let stack = BisectStack::build();
    let repo = Repo::open(&stack.path).expect("open repo");
    let (good, bad) = ends(&repo);
    repo.start_bisect_between(&good, &bad)
        .expect("start bisect");
    (stack, repo)
}

fn ends(repo: &Repo) -> (String, String) {
    (
        find_change(repo, BisectStack::GOOD).change_id.id,
        find_change(repo, BisectStack::BAD_END).change_id.id,
    )
}

fn ops_starting_with(repo: &Repo, prefix: &str) -> usize {
    repo.op_log()
        .expect("op log")
        .iter()
        .filter(|op| op.description.starts_with(prefix))
        .count()
}

fn working_copy(repo: &Repo) -> jayjay_core::ChangeInfo {
    repo.log("@")
        .expect("log @")
        .into_iter()
        .next()
        .expect("working copy")
}

fn subject(description: &str) -> &str {
    description.lines().next().unwrap_or("").trim()
}

fn mark_until_done(repo: &Repo, skip_before_bad: bool) -> (u32, BisectState) {
    let mut steps = 0;
    loop {
        let state = repo.bisect_state().expect("session");
        if state.phase == BisectPhase::Finished {
            return (steps, state);
        }
        assert_eq!(state.phase, BisectPhase::Evaluating, "{}", state.status);
        let candidate = state.candidate.expect("candidate");
        let name = subject(&candidate.description);
        let number: u32 = name
            .strip_prefix("change-")
            .unwrap_or_else(|| panic!("unexpected candidate {name}"))
            .parse()
            .expect("change number");
        let verdict = if skip_before_bad && name == BisectStack::BEFORE_BAD {
            BisectVerdict::Skip
        } else if number >= 5 {
            BisectVerdict::Bad
        } else {
            BisectVerdict::Good
        };
        steps += 1;
        assert!(steps < 20, "bisect did not finish");
        repo.bisect_mark(&candidate.commit_id, verdict)
            .expect("mark candidate");
    }
}

#[test]
fn manual_marking_finds_the_known_bad_change_and_restores_the_working_copy() {
    let stack = BisectStack::build();
    let repo = Repo::open(&stack.path).expect("open repo");
    let original = find_change(&repo, BisectStack::BAD_END);
    let original_id = original.change_id.id.clone();
    assert!(
        original.is_working_copy,
        "change-8 starts as the working copy"
    );
    let good = find_change(&repo, BisectStack::GOOD).change_id.id;
    repo.start_bisect_between(&good, &original_id)
        .expect("start bisect");

    let (steps, state) = mark_until_done(&repo, false);

    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::Found);
    assert!(
        result.bad[0].description.contains(BisectStack::FIRST_BAD),
        "{}",
        result.bad[0].description
    );
    let range = 8u32;
    let expected = (range as f64).log2().ceil() as u32;
    assert!(
        steps + 1 >= expected && steps <= expected + 1,
        "took {steps} steps, expected {expected} ± 1"
    );
    let restored = find_change(&repo, BisectStack::BAD_END);
    assert_eq!(restored.change_id.id, original_id);
    assert!(restored.is_working_copy);
    assert!(result.kept.is_empty());
}

#[test]
fn an_empty_working_copy_restores_to_a_new_change_on_the_same_parents() {
    let stack = BisectStack::build();
    run_jj_in(&stack.path, &["new"]);
    let repo = Repo::open(&stack.path).expect("open repo");
    let original = working_copy(&repo);
    assert!(original.is_empty, "the fresh working copy should be empty");
    assert!(original.description.trim().is_empty());
    let parents = original.parents.clone();
    let (good, bad) = ends(&repo);
    repo.start_bisect_between(&good, &bad).expect("start");

    let (_steps, state) = mark_until_done(&repo, false);

    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::Found);
    assert!(
        result.restore_note.is_none(),
        "a planned restore onto the original parents has nothing to explain: {:?}",
        result.restore_note
    );
    let restored = working_copy(&repo);
    assert_ne!(restored.change_id.id, original.change_id.id);
    assert_eq!(restored.parents, parents);
    assert!(restored.is_empty);
    assert!(restored.description.trim().is_empty());
}

#[test]
fn marking_a_non_candidate_or_the_candidate_twice_is_an_error() {
    let (_stack, repo) = started();
    let state = repo.bisect_state().expect("session");
    let candidate = state.candidate.expect("candidate").commit_id;
    let head = find_change(&repo, BisectStack::BAD_END).commit_id.id;
    assert_ne!(
        candidate, head,
        "the bad head is assumed bad and is not a candidate"
    );

    let rejected = repo.bisect_mark(&head, BisectVerdict::Good);
    assert!(
        rejected.is_err(),
        "marking the bad head good must not reach jj-lib"
    );
    assert_eq!(
        repo.bisect_state()
            .expect("session")
            .candidate
            .expect("candidate")
            .commit_id,
        candidate
    );

    repo.bisect_mark(&candidate, BisectVerdict::Good)
        .expect("first mark");
    let again = repo.bisect_mark(&candidate, BisectVerdict::Bad);
    assert!(again.is_err(), "a second mark must not reach jj-lib");
    let state = repo.bisect_state().expect("session");
    assert!(
        state
            .marks
            .iter()
            .any(|mark| { mark.commit_id == candidate && mark.verdict == BisectVerdict::Good })
    );
    assert_ne!(state.candidate.expect("next").commit_id, candidate);
}

#[test]
fn skipping_the_change_before_the_bad_one_reports_it_as_possibly_bad() {
    let (_stack, repo) = started();
    let (_steps, state) = mark_until_done(&repo, true);
    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::FoundDespiteSkips);
    assert!(
        result
            .possibly_bad
            .iter()
            .any(|change| change.description.contains(BisectStack::BEFORE_BAD)),
        "possibly bad: {:?}",
        result
            .possibly_bad
            .iter()
            .map(|change| subject(&change.description))
            .collect::<Vec<_>>()
    );
    assert!(result.bad[0].description.contains(BisectStack::FIRST_BAD));
}

#[test]
fn a_one_commit_range_is_found_without_checking_out() {
    let stack = BisectStack::build();
    let repo = Repo::open(&stack.path).expect("open");
    let original = working_copy(&repo).change_id.id;
    let parent = find_change(&repo, "change-7").change_id.id;
    let head = find_change(&repo, BisectStack::BAD_END).change_id.id;
    let state = repo.start_bisect_between(&parent, &head).expect("start");
    assert_eq!(state.phase, BisectPhase::Finished);
    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::Found);
    assert!(result.bad[0].description.contains(BisectStack::BAD_END));
    assert_eq!(working_copy(&repo).change_id.id, original);
    assert_eq!(ops_starting_with(&repo, "bisect: test"), 0);
}

#[test]
fn composing_a_bisect_range_treats_a_revset_and_two_empty_ends_the_same_way() {
    let missing_good = compose_bisect_range("", "bad", "").expect_err("missing good");
    let missing_bad = compose_bisect_range("good", "", "  ").expect_err("missing bad");
    assert_eq!(missing_good.to_string(), missing_bad.to_string());
    assert!(matches!(missing_good, JayError::Refused { .. }));
    assert_eq!(
        compose_bisect_range(" trunk() ", "abc", "").expect("ends"),
        "(trunk())..(abc)"
    );
    assert_eq!(
        compose_bisect_range("ignored", "ignored", " none() ").expect("revset"),
        "none()"
    );
}

#[test]
fn an_empty_range_or_unrelated_changes_are_refused() {
    let stack = BisectStack::build();
    let repo = Repo::open(&stack.path).expect("open");
    let empty = repo.start_bisect("none()");
    assert!(empty.is_err());
    assert!(repo.bisect_state().is_none());

    let good = find_change(&repo, BisectStack::GOOD).change_id.id;
    repo.new_change(&good, "side")
        .expect("sibling of the stack");
    let side = find_change(&repo, "side").change_id.id;
    let bad = find_change(&repo, BisectStack::BAD_END).change_id.id;
    let unrelated = repo.start_bisect_between(&side, &bad);
    assert!(unrelated.is_err());
    assert!(repo.bisect_state().is_none());
}

#[test]
fn a_temporary_change_with_content_is_kept() {
    let (stack, repo) = started();
    fs::write(stack.path.join("scratch.txt"), "notes while testing\n").expect("write scratch");
    let (_steps, state) = mark_until_done(&repo, false);
    let kept = state.result.expect("result").kept;
    assert_eq!(kept.len(), 1);
    let paths = changed_paths(&repo, &kept[0].change_id);
    assert!(paths.iter().any(|path| path == "scratch.txt"), "{paths:?}");
}

#[test]
fn rewriting_a_range_commit_pauses_the_session() {
    let (_stack, repo) = started();
    let candidate = repo
        .bisect_state()
        .expect("session")
        .candidate
        .expect("candidate")
        .commit_id;
    let victim = find_change(&repo, "change-1");
    repo.describe(&victim.change_id.id, "change-1 rewritten")
        .expect("rewrite a range commit");

    let state = repo
        .bisect_mark(&candidate, BisectVerdict::Good)
        .expect("mark");

    assert_eq!(state.phase, BisectPhase::PausedRangeChanged);
    assert!(
        state.marks.is_empty(),
        "the mark must not land once the range is gone"
    );
    assert!(repo.bisect_state().is_some());
}

#[cfg(unix)]
#[test]
fn a_command_finds_the_bad_change_and_maps_skip_and_abort() {
    let (_stack, repo) = started();
    let sync = repo.sync_token();
    let state = repo
        .bisect_run_to_end("if [ -f bug.txt ]; then exit 1; else exit 0; fi", &sync)
        .expect("run")
        .expect("session");
    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::Found);
    assert!(result.bad[0].description.contains(BisectStack::FIRST_BAD));

    let (_stack, repo) = started();
    let sync = repo.sync_token();
    let state = repo
        .bisect_run_to_end("if [ -f bug.txt ]; then exit 1; else exit 125; fi", &sync)
        .expect("run")
        .expect("session");
    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::FoundDespiteSkips);
    assert!(
        state
            .marks
            .iter()
            .any(|mark| mark.verdict == BisectVerdict::Skip)
    );
    assert!(!result.possibly_bad.is_empty());

    let (_stack, repo) = started();
    let sync = repo.sync_token();
    let state = repo
        .bisect_run_step("exit 127", &sync)
        .expect("run")
        .expect("session");
    let result = state.result.expect("result");
    assert_eq!(result.kind, BisectResultKind::Abort);
    assert_eq!(
        result.message,
        "The test command returned 127 (command not found)."
    );
}

#[cfg(unix)]
#[test]
fn stop_during_a_command_waits_for_it_to_exit_without_marking_the_candidate() {
    use std::thread;
    use std::time::{Duration, Instant};

    let (stack, repo) = started();
    let original = find_change(&repo, BisectStack::BAD_END).change_id.id;
    let pid_path = stack.path.join("bisect.pid");
    let marker = stack.path.join("bisect-running");
    let script = format!(
        "trap '' TERM; echo $$ > '{}'; touch '{}'; sleep 30",
        pid_path.display(),
        marker.display()
    );
    let sync = repo.sync_token();

    thread::scope(|scope| {
        let run = scope.spawn(|| repo.bisect_run_step(&script, &sync));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !marker.exists() {
            assert!(Instant::now() < deadline, "the test command never started");
            thread::sleep(Duration::from_millis(20));
        }
        let pid = fs::read_to_string(&pid_path).expect("pid");
        assert!(process_alive(&pid));
        let stop = scope.spawn(|| {
            sync.cancel();
            repo.bisect_stop()
        });
        stop.join().expect("stop thread").expect("stop");
        assert!(
            !process_alive(&pid),
            "bisect_stop returned while the test process was still alive"
        );
        assert!(run.join().expect("command thread").expect("run").is_none());
    });

    assert!(repo.bisect_state().is_none());
    assert_eq!(working_copy(&repo).change_id.id, original);
    assert_eq!(ops_starting_with(&repo, "bisect: test"), 1);
    assert_eq!(ops_starting_with(&repo, "bisect: finish"), 1);
}

#[cfg(unix)]
fn process_alive(pid: &str) -> bool {
    std::process::Command::new("kill")
        .args(["-0", pid.trim()])
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
fn marking_or_testing_after_the_working_copy_moves_records_nothing() {
    for same_parent in [false, true] {
        let (_stack, repo) = started();
        let before = repo.bisect_state().expect("session");
        let candidate = before.candidate.as_ref().expect("candidate");
        if same_parent {
            repo.new_change(&candidate.commit_id, "")
                .expect("new change on the same candidate");
        } else {
            let bad = find_change(&repo, BisectStack::BAD_END).change_id.id;
            repo.edit(&bad).expect("edit another change");
        }

        assert!(matches!(
            repo.bisect_mark(&candidate.commit_id, BisectVerdict::Bad),
            Err(JayError::Refused { .. })
        ));
        #[cfg(unix)]
        {
            assert!(matches!(
                repo.bisect_run_step("touch .jj/bisect-tested; exit 1", &repo.sync_token()),
                Err(JayError::Refused { .. })
            ));
            assert!(!_stack.path.join(".jj/bisect-tested").exists());
        }
        assert_eq!(repo.bisect_state().expect("session"), before);
        repo.bisect_stop().expect("stop");
    }
}

#[cfg(unix)]
#[test]
fn a_command_verdict_is_rejected_if_the_working_copy_moves_while_running() {
    use std::thread;
    use std::time::{Duration, Instant};

    let (stack, repo) = started();
    let before = repo.bisect_state().expect("session");
    let bad = find_change(&repo, BisectStack::BAD_END).change_id.id;
    let running = stack.path.join(".jj/bisect-running");
    let finish = stack.path.join(".jj/bisect-finish");
    let sync = repo.sync_token();
    thread::scope(|scope| {
        let command = scope.spawn(|| {
            repo.bisect_run_step(
                "touch .jj/bisect-running; while [ ! -f .jj/bisect-finish ]; do sleep 0.02; done; exit 1",
                &sync,
            )
        });
        let deadline = Instant::now() + Duration::from_secs(10);
        while !running.exists() {
            if Instant::now() >= deadline {
                sync.cancel();
                panic!("test command never started");
            }
            thread::sleep(Duration::from_millis(20));
        }
        let moved = repo.edit(&bad);
        fs::write(&finish, "finish").expect("finish test command");
        moved.expect("edit while testing");
        assert!(matches!(
            command.join().expect("command thread"),
            Err(JayError::Refused { .. })
        ));
    });
    assert_eq!(repo.bisect_state().expect("session"), before);
    repo.bisect_stop().expect("stop");
}

#[test]
fn an_empty_working_copy_in_the_range_stays_visible_and_is_restored() {
    for between in [false, true] {
        let stack = BisectStack::build();
        run_jj_in(&stack.path, &["new"]);
        let repo = Repo::open(&stack.path).expect("open");
        let original = working_copy(&repo);
        let before = repo.log("all()").expect("log").len();
        let good = find_change(&repo, BisectStack::GOOD).change_id.id;
        if between {
            repo.start_bisect_between(&good, &original.change_id.id)
        } else {
            repo.start_bisect(&format!("({good})..@"))
        }
        .expect("start with empty bad end");

        let (_, state) = mark_until_done(&repo, false);

        let result = state.result.expect("result");
        assert_eq!(result.kind, BisectResultKind::Found);
        assert!(result.bad[0].description.contains(BisectStack::FIRST_BAD));
        assert_eq!(working_copy(&repo).change_id.id, original.change_id.id);
        assert_eq!(repo.log("all()").expect("log").len(), before);
        assert!(result.kept.is_empty());
    }
}

#[test]
fn shutdown_racing_a_start_always_leaves_the_working_copy_restored() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    for delay_ms in (0..24).step_by(2) {
        let stack = BisectStack::build();
        let repo = Arc::new(Repo::open(&stack.path).expect("open"));
        let original = working_copy(&repo).change_id.id;
        let good = find_change(&repo, BisectStack::GOOD).change_id.id;
        let bad = find_change(&repo, BisectStack::BAD_END).change_id.id;
        let barrier = Arc::new(Barrier::new(2));

        let start = {
            let (repo, barrier) = (repo.clone(), barrier.clone());
            thread::spawn(move || {
                barrier.wait();
                repo.start_bisect_between(&good, &bad).map(|_| ())
            })
        };
        barrier.wait();
        thread::sleep(Duration::from_millis(delay_ms));
        repo.shutdown_bisect().expect("shutdown");
        let started = start.join().expect("start thread");

        assert!(
            matches!(started, Ok(()) | Err(JayError::Canceled)),
            "{started:?}"
        );
        assert!(repo.bisect_state().is_none());
        assert_eq!(working_copy(&repo).change_id.id, original);
    }
}

#[test]
fn restoration_follows_rewritten_parents_without_resurrecting_old_commits() {
    for original_was_described in [false, true] {
        for restart in [false, true] {
            let stack = BisectStack::build();
            run_jj_in(
                &stack.path,
                &[
                    "new",
                    "-m",
                    if original_was_described {
                        "original"
                    } else {
                        ""
                    },
                ],
            );
            let repo = Repo::open(&stack.path).expect("open");
            let original = working_copy(&repo);
            let good = find_change(&repo, BisectStack::GOOD).change_id.id;
            let bad = find_change(&repo, BisectStack::BAD_END);
            repo.start_bisect_between(&good, &bad.change_id.id)
                .expect("start");
            repo.describe(&bad.change_id.id, "rewritten bad end")
                .expect("rewrite parent");
            let rewritten = find_change(&repo, "rewritten bad end");
            assert_ne!(rewritten.commit_id.id, bad.commit_id.id);
            if original_was_described {
                repo.abandon(&original.change_id.id)
                    .expect("abandon original");
            }
            let candidate = repo
                .bisect_state()
                .expect("session")
                .candidate
                .expect("candidate");
            assert_eq!(
                repo.bisect_mark(&candidate.commit_id, BisectVerdict::Good)
                    .expect("pause")
                    .phase,
                BisectPhase::PausedRangeChanged
            );
            assert!(repo.log("divergent()").expect("before restore").is_empty());

            if restart {
                repo.bisect_restart().expect("restart");
                let (_, state) = mark_until_done(&repo, false);
                assert_eq!(state.result.expect("result").kind, BisectResultKind::Found);
            } else {
                repo.bisect_stop().expect("stop");
            }

            assert_eq!(working_copy(&repo).parents, vec![rewritten.commit_id.id]);
            assert!(repo.log("divergent()").expect("after restore").is_empty());
        }
    }
}

#[test]
fn stopping_after_returning_to_the_original_empty_change_preserves_it() {
    let stack = BisectStack::build();
    run_jj_in(&stack.path, &["new"]);
    let repo = Repo::open(&stack.path).expect("open");
    let original = working_copy(&repo);
    let good = find_change(&repo, BisectStack::GOOD).change_id.id;
    repo.start_bisect_between(&good, &original.change_id.id)
        .expect("start");
    repo.edit(&original.change_id.id)
        .expect("return to original");
    assert_eq!(working_copy(&repo).change_id.id, original.change_id.id);

    repo.bisect_stop().expect("stop");

    let restored = working_copy(&repo);
    assert_eq!(restored.change_id.id, original.change_id.id);
    assert_eq!(restored.commit_id.id, original.commit_id.id);
    assert!(repo.bisect_state().is_none());
}
