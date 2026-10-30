use jayjay_network::{MockFetch, NetError, Request, Response};
use serde_json::json;
use tempfile::TempDir;

use super::*;
use crate::repo::forge_auth::{ForgeToken, TokenSource};

fn new_store(path: &std::path::Path) -> FlockStore {
    FlockStore(JsonStore::new(
        "flock",
        Some(path.to_owned()),
        State::normalize,
    ))
}

fn tokens() -> Option<ForgeToken> {
    Some(ForgeToken {
        secret: "private-token".into(),
        source: TokenSource::Stored,
    })
}

fn answer(
    id: &str,
    login: &str,
    name: Option<&str>,
    sponsored: bool,
) -> MockFetch<impl Fn(&Request) -> Result<Response, NetError>> {
    let body = json!({"data": {
        "viewer": {"id": id, "login": login, "name": name},
        "user": {"sponsorshipForViewerAsSponsor": sponsored.then(|| json!({"__typename": "Sponsorship"}))}
    }})
    .to_string();
    MockFetch::new(move |_| Response::ok(200, body.clone()))
}

#[test]
fn sponsor_code_unlocks_only_with_a_matching_code_and_a_name() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("flock.json");
    let mut store = new_store(&path);
    let expected = hex::encode(Sha256::digest("FLOCK-TEST"));
    for (code, name) in [("FLOCK-WRONG", "Supporter"), ("flock-test", "  ")] {
        assert_eq!(store.redeem_with(code, name, &expected).unwrap(), None);
    }
    assert!(!path.exists());

    let member = store
        .redeem_with(" flock-test\n", " Supporter ", &expected)
        .unwrap()
        .unwrap();
    assert_eq!(member.login, "Supporter");
    assert_eq!(new_store(&path).member(), Some(member));
    assert!(
        !std::fs::read_to_string(&path)
            .unwrap()
            .contains("FLOCK-TEST")
    );
}

#[test]
fn verified_identity_persists_offline_without_keeping_the_token() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("flock.json");
    let mut store = new_store(&path);
    let fetch = answer("U_sponsor", "supporter", Some("First Supporter"), true);
    let sync = SyncToken::standalone();
    let FlockVerification::Unlocked { member } = store
        .verify_with("another-maintainer", &fetch, &tokens(), &sync)
        .unwrap()
    else {
        panic!("expected an unlock");
    };
    assert_eq!(member.id, "U_sponsor");
    assert_eq!(member.name.as_deref(), Some("First Supporter"));
    let request = &fetch.requests()[0];
    assert!(
        request
            .headers
            .iter()
            .any(|header| header.name == "Authorization")
    );
    let body: serde_json::Value = serde_json::from_str(request.body_text()).unwrap();
    assert_eq!(body["variables"]["sponsorable"], "another-maintainer");
    let query = body["query"].as_str().unwrap();
    assert!(!query.contains("another-maintainer"));
    assert!(query.contains("activeOnly: false"));
    assert_eq!(new_store(&path).member(), Some(member.clone()));
    assert!(
        !std::fs::read_to_string(&path)
            .unwrap()
            .contains("private-token")
    );

    let offline = MockFetch::new(|_| Err(NetError::Transport));
    assert!(
        store
            .verify_with("another-maintainer", &offline, &tokens(), &sync)
            .is_err()
    );
    assert_eq!(store.member(), Some(member));

    std::fs::write(&path, r#"{"member":{"id":"U_legacy","login":"legacy"}}"#).unwrap();
    let legacy = new_store(&path).member().unwrap();
    assert_eq!(legacy.login, "legacy");
    assert_eq!(legacy.name, None);
}

#[test]
fn no_failed_or_negative_check_changes_the_verified_account() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("flock.json");
    let mut store = new_store(&path);
    let sync = SyncToken::standalone();
    store
        .verify_with(
            "another-maintainer",
            &answer("U_one", "first", None, true),
            &tokens(),
            &sync,
        )
        .unwrap();
    let original = store.member();
    let failures = [
        (403, "{}"),
        (429, "{}"),
        (502, "<html>"),
        (200, "{}"),
        (
            200,
            r#"{"data":{"viewer":{"id":"U_other","login":"other"},"user":null}}"#,
        ),
        (
            200,
            r#"{"data":{"viewer":{"id":"U_other","login":"other"},"user":{"sponsorshipForViewerAsSponsor":null}},"errors":[{"message":"forbidden"}]}"#,
        ),
    ];
    for (status, body) in failures {
        let fetch = MockFetch::new(move |_| Response::ok(status, body));
        assert!(
            store
                .verify_with("another-maintainer", &fetch, &tokens(), &sync)
                .is_err(),
            "{status}: {body}"
        );
        assert_eq!(store.member(), original);
    }
    for (status, body) in [
        (401, "{}"),
        (
            200,
            r#"{"data":null,"errors":[{"type":"INSUFFICIENT_SCOPES"}]}"#,
        ),
        (
            200,
            r#"{"data":{"viewer":null,"user":null},"errors":[{"type":"INSUFFICIENT_SCOPES"}]}"#,
        ),
        (200, r#"{"data":null,"errors":[{"type":"FORBIDDEN"}]}"#),
    ] {
        let fetch = MockFetch::new(move |_| Response::ok(status, body));
        assert_eq!(
            store
                .verify_with("another-maintainer", &fetch, &tokens(), &sync)
                .unwrap(),
            FlockVerification::SignInRequired
        );
        assert_eq!(store.member(), original);
    }
    let no_login = MockFetch::new(|_| panic!("no login must not request GitHub"));
    assert_eq!(
        store
            .verify_with("another-maintainer", &no_login, &None, &sync)
            .unwrap(),
        FlockVerification::SignInRequired
    );
    assert_eq!(store.member(), original);

    assert_eq!(
        store
            .verify_with(
                "another-maintainer",
                &answer("U_other", "other", None, false),
                &tokens(),
                &sync
            )
            .unwrap(),
        FlockVerification::NotSponsored {
            login: "other".into()
        }
    );
    assert_eq!(store.member(), original);
    assert_eq!(
        store
            .verify_existing_login(
                "another-maintainer",
                &answer("U_other", "other", None, false),
                &tokens(),
                &sync
            )
            .unwrap(),
        FlockVerification::SignInRequired
    );
    assert_eq!(store.member(), original);
    let FlockVerification::Unlocked { member } = store
        .verify_existing_login(
            "another-maintainer",
            &answer("U_two", "second", None, true),
            &tokens(),
            &sync,
        )
        .unwrap()
    else {
        panic!("a visible sponsorship needs no further sign-in");
    };
    assert_eq!(store.member(), Some(member));

    store.forget().unwrap();
    assert_eq!(new_store(&path).member(), None);
}

#[test]
fn cancellation_and_failed_persistence_do_not_publish_an_unlock() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("flock.json");
    let mut store = new_store(&path);
    let sync = SyncToken::standalone();
    let fetch = MockFetch::new(|_| {
        sync.cancel();
        Response::ok(
            200,
            r#"{"data":{"viewer":{"id":"U_sponsor","login":"supporter"},"user":{"sponsorshipForViewerAsSponsor":{"__typename":"Sponsorship"}}}}"#,
        )
    });
    assert!(matches!(
        store.verify_with("another-maintainer", &fetch, &tokens(), &sync),
        Err(JayError::Canceled)
    ));
    assert_eq!(store.member(), None);
    assert!(!path.exists());

    let blocked = dir.path().join("blocked");
    std::fs::write(&blocked, "file, not directory").unwrap();
    let mut blocked_store = new_store(&blocked.join("flock.json"));
    assert!(
        blocked_store
            .verify_with(
                "another-maintainer",
                &answer("U_sponsor", "supporter", Some("First Supporter"), true),
                &tokens(),
                &SyncToken::standalone()
            )
            .is_err()
    );
    assert_eq!(blocked_store.member(), None);
}
