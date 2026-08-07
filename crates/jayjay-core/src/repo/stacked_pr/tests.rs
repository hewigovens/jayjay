use super::result_open_urls;
use crate::repo::hosted_repo::RepoHost;
use crate::types::{StackLayerOutcome, SubmittedLayer};

fn layer(number: u32, url: &str) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: format!("layer-{number}"),
        base: "main".to_owned(),
        title: format!("Layer {number}"),
        outcome: StackLayerOutcome::Created,
        pr_number: number,
        pr_url: url.to_owned(),
        detail: "ready".to_owned(),
    }
}

#[test]
fn linked_native_stack_opens_only_the_top_pull_request() {
    let layers = [
        layer(10, "https://example.test/pull/10"),
        layer(11, "https://example.test/pull/11"),
        layer(12, "https://example.test/pull/12"),
    ];

    assert_eq!(
        result_open_urls(&layers, RepoHost::GitHub, true),
        ["https://example.test/pull/12"]
    );
}

#[test]
fn github_dependent_chain_opens_every_available_pull_request() {
    let layers = [
        layer(10, "https://example.test/pull/10"),
        layer(11, ""),
        layer(12, "https://example.test/pull/12"),
    ];

    assert_eq!(
        result_open_urls(&layers, RepoHost::GitHub, false),
        [
            "https://example.test/pull/10",
            "https://example.test/pull/12"
        ]
    );
}

#[test]
fn gitlab_stack_opens_only_the_top_merge_request() {
    let layers = [
        layer(10, "https://example.test/merge_requests/10"),
        layer(11, "https://example.test/merge_requests/11"),
        layer(12, "https://example.test/merge_requests/12"),
    ];

    assert_eq!(
        result_open_urls(&layers, RepoHost::GitLab, false),
        ["https://example.test/merge_requests/12"]
    );
}

#[test]
fn gitlab_result_opens_the_highest_available_merge_request() {
    let mut failed = layer(12, "");
    failed.outcome = StackLayerOutcome::Failed;
    failed.pr_number = 0;
    let layers = [
        layer(10, "https://example.test/merge_requests/10"),
        layer(11, "https://example.test/merge_requests/11"),
        failed,
    ];

    assert_eq!(
        result_open_urls(&layers, RepoHost::GitLab, false),
        ["https://example.test/merge_requests/11"]
    );
}

#[test]
fn linked_stack_without_a_top_url_preserves_dependent_chain_fallback() {
    let layers = [layer(10, "https://example.test/pull/10"), layer(11, "")];

    assert_eq!(
        result_open_urls(&layers, RepoHost::GitHub, true),
        ["https://example.test/pull/10"]
    );
}
