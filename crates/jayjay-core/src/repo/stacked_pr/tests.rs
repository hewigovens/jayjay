use super::stack_forge::StackForge;
use crate::repo::hosted_repo::HostedRepo;
use crate::types::{StackLayerOutcome, SubmittedLayer};

fn layer(number: u32, url: &str) -> SubmittedLayer {
    SubmittedLayer {
        bookmark: format!("layer-{number}"),
        base: "main".to_owned(),
        title: format!("Layer {number}"),
        outcome: if url.is_empty() {
            StackLayerOutcome::Failed
        } else {
            StackLayerOutcome::Created
        },
        pr_number: if url.is_empty() { 0 } else { number },
        pr_url: url.to_owned(),
        detail: String::new(),
    }
}

fn open_urls(remote: &str, layers: &[SubmittedLayer], linked: bool) -> Vec<String> {
    StackForge::for_remote(HostedRepo::parse(remote))
        .unwrap()
        .open_urls(layers, linked)
}

#[test]
fn done_opens_the_top_pull_request_only_when_it_reaches_the_whole_stack() {
    let all = [layer(10, "u10"), layer(11, "u11"), layer(12, "u12")];
    let failed_top = [layer(10, "u10"), layer(11, "u11"), layer(12, "")];
    let github = "https://github.com/o/r";
    let gitlab = "https://gitlab.com/o/r";
    let codeberg = "https://codeberg.org/o/r";

    assert_eq!(open_urls(github, &all, true), ["u12"]);
    assert_eq!(open_urls(github, &all, false), ["u10", "u11", "u12"]);
    assert_eq!(open_urls(github, &failed_top, true), ["u10", "u11"]);
    assert_eq!(open_urls(gitlab, &all, false), ["u12"]);
    assert_eq!(open_urls(gitlab, &failed_top, false), ["u11"]);
    assert_eq!(open_urls(codeberg, &all, true), ["u12"]);
    assert_eq!(open_urls(codeberg, &failed_top, false), ["u10", "u11"]);
}
