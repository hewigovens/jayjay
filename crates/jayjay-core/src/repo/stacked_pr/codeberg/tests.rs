use jayjay_network::{Method, MockFetch, NetError, Response};

use super::super::forge::ForgeTarget;
use super::section::{with_existing_section, with_stack_section};
use super::stack::CodebergStack;
use crate::repo::hosted_repo::HostedRepo;
use crate::repo::pull_requests::codeberg::PAGE_SIZE;
use crate::types::{StackLayerOutcome, SubmittedLayer};

const REPO: &str = "https://codeberg.org/api/v1/repos/owner/repo";

fn pr(number: u32, head: &str, body: &str) -> String {
    format!(
        r#"{{"number":{number},"state":"open","body":{body:?},
           "html_url":"https://codeberg.org/owner/repo/pulls/{number}",
           "head":{{"ref":"{head}","repo":{{"full_name":"owner/repo"}}}}}}"#
    )
}

fn page(prs: impl IntoIterator<Item = String>) -> String {
    format!("[{}]", prs.into_iter().collect::<Vec<_>>().join(","))
}

fn full_page() -> String {
    page((0..PAGE_SIZE).map(|i| pr(i, "other", "")))
}

fn remote() -> HostedRepo {
    HostedRepo::parse("https://codeberg.org/owner/repo").unwrap()
}

fn target(bookmark: &str, base: &str) -> ForgeTarget {
    ForgeTarget {
        bookmark: bookmark.to_owned(),
        base: base.to_owned(),
        title: format!("title for {bookmark}"),
        body: format!("body for {bookmark}"),
    }
}

fn submitted(number: u32) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: format!("layer-{number}"),
        base: "main".to_owned(),
        title: String::new(),
        outcome: StackLayerOutcome::Created,
        pr_number: number,
        pr_url: String::new(),
        detail: String::new(),
    }
}

#[test]
fn resubmission_updates_the_existing_pull_request_and_keeps_its_stack_section() {
    let section = with_stack_section("old", 7, &[6, 7]);
    let fetch = MockFetch::new(move |request| match request.method {
        Method::Get if request.url.contains("page=1&") => Response::ok(200, full_page()),
        Method::Get => Response::ok(200, page([pr(7, "feat", &section)])),
        _ => Response::ok(200, "{}"),
    });
    let remote = remote();

    let layer =
        CodebergStack::new(&fetch, &remote, Some("t".into())).submit_layer(&target("feat", "main"));

    assert_eq!(
        (layer.outcome, layer.pr_number),
        (StackLayerOutcome::Updated, 7)
    );
    let writes = fetch.writes();
    assert_eq!(writes.len(), 1, "no create may race the existing PR");
    let (method, url, body) = (writes[0].method, &writes[0].url, writes[0].body_text());
    assert_eq!(method, Method::Patch);
    assert_eq!(*url, format!("{REPO}/pulls/7"));
    assert!(
        body.contains(r#""base":"main""#) && !body.contains("head"),
        "{body}"
    );
    assert!(body.contains("#6 → **#7**"), "{body}");
}

#[test]
fn an_absent_pull_request_is_created_from_the_bookmark() {
    let fetch = MockFetch::new(|request| match request.method {
        Method::Get => Response::ok(200, "[]"),
        _ => Response::ok(201, pr(20, "feat", "")),
    });
    let remote = remote();

    let layer =
        CodebergStack::new(&fetch, &remote, Some("t".into())).submit_layer(&target("feat", "main"));

    assert_eq!(
        (layer.outcome, layer.pr_number),
        (StackLayerOutcome::Created, 20)
    );
    assert_eq!(layer.pr_url, "https://codeberg.org/owner/repo/pulls/20");
    let writes = fetch.writes();
    let (method, body) = (writes[0].method, writes[0].body_text());
    assert_eq!(method, Method::Post);
    assert!(body.contains(r#""head":"feat","base":"main""#), "{body}");
}

#[test]
fn an_inconclusive_lookup_fails_the_layer_without_writing() {
    let failed_page_two = MockFetch::new(|request| {
        if request.url.contains("page=1&") {
            Response::ok(200, full_page())
        } else {
            Err(NetError::Transport)
        }
    });
    let capped = MockFetch::new(|_| Response::ok(200, full_page()));
    let remote = remote();

    for layer in [
        CodebergStack::new(&failed_page_two, &remote, Some("t".into()))
            .submit_layer(&target("feat", "main")),
        CodebergStack::new(&capped, &remote, Some("t".into()))
            .submit_layer(&target("feat", "main")),
    ] {
        assert_eq!(layer.outcome, StackLayerOutcome::Failed, "{layer:?}");
    }
    assert!(failed_page_two.writes().is_empty());
    assert!(capped.writes().is_empty());
}

#[test]
fn linking_writes_every_layer_or_none() {
    let targets = [target("a", "main"), target("b", "a"), target("c", "b")];
    let remote = remote();

    let fetch = MockFetch::new(|_| Response::ok(200, "{}"));
    let linked = CodebergStack::new(&fetch, &remote, Some("t".into()))
        .link(&targets, &[submitted(10), submitted(11), submitted(12)]);
    assert!(linked.is_ok(), "{linked:?}");
    let writes = fetch.writes();
    assert_eq!(writes.len(), 3);
    assert!(
        writes[0].body_text().contains("**#10** → #11 → #12"),
        "{}",
        writes[0].body_text()
    );

    let fetch = MockFetch::new(|_| Response::ok(200, "{}"));
    let failed = SubmittedLayer {
        outcome: StackLayerOutcome::Failed,
        pr_number: 0,
        ..submitted(0)
    };
    let notes = CodebergStack::new(&fetch, &remote, Some("t".into()))
        .link(&targets, &[submitted(10), failed, submitted(12)])
        .expect_err("a failed layer leaves the links stale");
    assert!(notes.contains("submit again"), "{notes}");
    assert!(fetch.writes().is_empty());

    let fetch = MockFetch::new(|_| Err(NetError::Transport));
    let notes = CodebergStack::new(&fetch, &remote, Some("t".into()))
        .link(&targets[..2], &[submitted(10), submitted(11)])
        .expect_err("rewrite failures are reported");
    assert!(
        notes.contains("PR #10") && notes.contains("PR #11"),
        "{notes}"
    );
}

#[test]
fn the_stack_section_is_replaced_never_appended() {
    let first = with_stack_section("Describes the change.", 12, &[10, 11, 12]);
    let again = with_stack_section(&first, 11, &[10, 11, 12]);
    let kept = with_existing_section("New text.", &again);

    for body in [&first, &again, &kept] {
        assert_eq!(body.matches("<!-- jayjay-stack -->").count(), 1, "{body}");
    }
    assert!(first.starts_with("Describes the change.") && first.contains("#11 → **#12**"));
    assert!(again.contains("#10 → **#11** → #12"), "{again}");
    assert!(
        kept.starts_with("New text.") && kept.contains("**#11**"),
        "{kept}"
    );
    assert_eq!(
        with_existing_section("New text.", "No links yet."),
        "New text."
    );
}

#[test]
fn preflight_names_the_failing_precondition_without_leaking_the_token() {
    const TOKEN: &str = "secret-token-1234";
    let remote = remote();
    let cases = [
        (None, 200, "{}", "need a token"),
        (
            Some(TOKEN),
            401,
            r#"{"message":"bad"}"#,
            "token was rejected",
        ),
        (
            Some(TOKEN),
            404,
            r#"{"message":"Not Found"}"#,
            "not found or the token cannot see it",
        ),
        (
            Some(TOKEN),
            200,
            r#"{"default_branch":"main","permissions":{"push":false}}"#,
            "cannot push",
        ),
        (
            Some(TOKEN),
            200,
            r#"{"permissions":{"push":true}}"#,
            "does not report a default branch",
        ),
    ];
    for (token, status, body, expected) in cases {
        let fetch = MockFetch::new(move |_| Response::ok(status, body));
        let error = CodebergStack::new(&fetch, &remote, token.map(str::to_owned))
            .preflight()
            .expect_err(expected)
            .to_string();
        assert!(
            error.contains(expected) && !error.contains(TOKEN),
            "{error}"
        );
    }

    let fetch = MockFetch::new(|_| {
        Response::ok(
            200,
            r#"{"default_branch":"trunk","permissions":{"push":true}}"#,
        )
    });
    let base = CodebergStack::new(&fetch, &remote, Some(TOKEN.into())).preflight();
    assert_eq!(base.unwrap(), "trunk");
}
