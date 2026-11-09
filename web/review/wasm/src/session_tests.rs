use serde_json::{Value, json};

use crate::session::ReviewSession;

fn session(identity: &str, after: &str) -> ReviewSession {
    ReviewSession::parse(&json!({
        "schemaVersion": 1, "repositoryId": "repo", "changeId": "change", "title": "Review",
        "files": [{"path": "a.txt", "reviewIdentity": identity, "before": "foo\nbar\n", "after": after}]
    }).to_string()).unwrap()
}

#[test]
fn note_actions_preserve_display_mode_native_schema_and_scope() {
    let session = session("v1", "foo \nfoo\nbar\n");
    let initial = json!({"reviewed": {}, "notes": [], "future": {"preserve": true}}).to_string();
    let state = session.mutate(&json!({"kind": "addNote", "file": 0, "side": "new", "line": 2, "body": "Inspect duplicate", "ignoreWhitespace": true}).to_string(), &initial).unwrap();
    let stored: Value = serde_json::from_str(&state).unwrap();
    let id = stored["notes"][0]["id"].as_str().unwrap();
    let edited = session
        .mutate(
            &json!({"kind": "updateNote", "id": id, "body": "Edited note"}).to_string(),
            &state,
        )
        .unwrap();
    let edited: Value = serde_json::from_str(&edited).unwrap();
    assert_eq!(edited["notes"][0]["body"], "Edited note");
    for field in [
        "id",
        "path",
        "line",
        "side",
        "anchor_excerpt",
        "ignore_whitespace",
        "created_at_ms",
        "resolved",
    ] {
        assert_eq!(edited["notes"][0][field], stored["notes"][0][field]);
    }
    for action in [
        json!({"kind": "updateNote", "id": id, "body": "  "}),
        json!({"kind": "updateNote", "id": "outside-change", "body": "Edited"}),
    ] {
        assert!(session.mutate(&action.to_string(), &state).is_err());
    }
    assert_eq!(stored["future"]["preserve"], true);
    assert_eq!(stored["notes"][0]["ignore_whitespace"], true);
    assert_eq!(
        session.describe(&state).unwrap()["notes"][0]["status"],
        "current"
    );
    assert!(
        session
            .mutate(r#"{"kind":"resolveNote","id":"outside-change"}"#, &state)
            .is_err()
    );
    assert!(session.mutate(&json!({"kind": "addNote", "file": 0, "side": "new", "line": 2, "body": "wrong mode", "ignoreWhitespace": false}).to_string(), &initial).is_err());
}

#[test]
fn revised_content_invalidates_marks_and_notes_across_serialized_sessions() {
    let first = session("v1", "different\n");
    let marked = first
        .mutate(
            r#"{"kind":"setFile","file":0,"reviewed":true}"#,
            r#"{"reviewed":{},"notes":[]}"#,
        )
        .unwrap();
    assert_eq!(
        first
            .mutate(r#"{"kind":"setFile","file":0,"reviewed":true}"#, &marked)
            .unwrap(),
        marked
    );
    let noted = first.mutate(r#"{"kind":"addNote","file":0,"side":"new","line":1,"body":"Check","ignoreWhitespace":false}"#, &marked).unwrap();
    assert_eq!(
        first.describe(&noted).unwrap()["files"][0]["review"],
        "reviewed"
    );
    let revised = session("v2", "other\n").describe(&noted).unwrap();
    assert_eq!(revised["files"][0]["review"], "changed_since_review");
    assert_eq!(revised["notes"][0]["status"], "stale");
}

#[test]
fn overview_carries_the_author_avatar_source() {
    let bundle = |email: &str| {
        ReviewSession::parse(&json!({
            "schemaVersion": 1, "repositoryId": "repo", "changeId": "change", "title": "Review", "authorEmail": email,
            "files": [{"path": "a.txt", "reviewIdentity": "v1", "before": "a\n", "after": "b\n"}]
        }).to_string()).unwrap()
    };
    let overview = |email: &str| {
        bundle(email)
            .describe(r#"{"reviewed":{},"notes":[]}"#)
            .unwrap()["authorAvatar"]
            .clone()
    };
    assert_eq!(
        overview("49699333+dependabot[bot]@users.noreply.github.com"),
        json!({"kind": "gitHubBot", "value": "49699333"})
    );
    assert_eq!(overview(""), Value::Null);
}

#[test]
fn counts_use_the_catalog_plural_for_each_number() {
    let session = session("v1", "foo\nbaz\n");
    let empty = r#"{"reviewed":{},"notes":[]}"#;
    let counts = &session.describe(empty).unwrap()["counts"];
    assert_eq!(counts["files"], "1 file");
    assert_eq!(counts["filtered"], json!(["0 of 1 files", "1 of 1 files"]));
    let marked = session
        .mutate(r#"{"kind":"setFile","file":0,"reviewed":true}"#, empty)
        .unwrap();
    assert_eq!(
        session.describe(&marked).unwrap()["counts"]["reviewed"],
        "1/1 reviewed"
    );
}

#[test]
fn describes_each_file_with_its_seti_icon() {
    let described = session("v1", "x\n")
        .describe(r#"{"reviewed":{},"notes":[]}"#)
        .unwrap();
    let glyph = jayjay_primitives::seti::SetiGlyph::for_path("a.txt").0;
    assert_eq!(described["files"][0]["icon"]["glyph"], glyph.to_string());
}

#[test]
fn files_load_on_demand_and_keep_their_review_state() {
    let empty = r#"{"reviewed":{},"notes":[]}"#;
    let first = session("v1", "foo\nbaz\n");
    let state = first
        .mutate(r#"{"kind":"setFile","file":0,"reviewed":true}"#, empty)
        .unwrap();
    let state = first.mutate(r#"{"kind":"addNote","file":0,"side":"new","line":2,"body":"Check","ignoreWhitespace":false}"#, &state).unwrap();

    let lazy = |identity: &str| {
        ReviewSession::parse(&json!({
            "schemaVersion": 1, "repositoryId": "repo", "changeId": "change", "title": "Review",
            "files": [{"path": "a.txt", "reviewIdentity": identity, "kind": "removed", "added": 0, "removed": 2}]
        }).to_string()).unwrap()
    };
    let mut session = lazy("v1");
    let overview = session.describe(&state).unwrap();
    assert_eq!(overview["files"][0]["kind"], "removed");
    assert_eq!(overview["files"][0]["removed"], 2);
    assert_eq!(overview["files"][0]["review"], "reviewed");
    assert_eq!(overview["notes"][0]["status"], "current");
    assert_eq!(
        lazy("v2").describe(&state).unwrap()["notes"][0]["status"],
        "stale"
    );
    assert!(session.file_state(0, &state, false).is_err());
    assert!(
        session
            .mutate(r#"{"kind":"setFile","file":0,"reviewed":false}"#, &state)
            .is_err()
    );

    session.load(0, "foo\nbar\n", "foo\nbaz\n").unwrap();
    assert_eq!(
        session.describe(&state).unwrap()["notes"][0]["group_index"],
        0
    );
    assert!(session.file_state(0, &state, false).is_ok());
    assert!(session.load(0, "\0", "").is_err());
}

#[test]
fn notices_name_what_the_text_diff_leaves_out() {
    let session = ReviewSession::parse(
        &json!({
            "schemaVersion": 1, "repositoryId": "repo", "changeId": "change", "title": "Review",
            "files": [
                {"path": "logo.bin", "reviewIdentity": "1", "binary": true},
                {"path": "run.sh", "reviewIdentity": "2", "oldMode": "100644", "newMode": "100755"},
                {"path": "a.txt", "reviewIdentity": "3"}
            ]
        })
        .to_string(),
    )
    .unwrap();
    let files = &session.describe(r#"{"reviewed":{},"notes":[]}"#).unwrap()["files"];
    assert_eq!(files[0]["notice"], "Binary file not shown.");
    assert_eq!(
        files[1]["notice"],
        "File mode changed from 100644 to 100755."
    );
    assert_eq!(files[2]["notice"], Value::Null);
}
