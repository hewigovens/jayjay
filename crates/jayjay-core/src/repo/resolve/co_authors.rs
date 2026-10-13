use jj_lib::backend::Signature;
use jj_lib::commit::Commit as JjCommit;
use jj_lib::trailer::parse_description_trailers;

use crate::github_alias;
use crate::types::CommitAuthor;

const CO_AUTHORED_BY: &str = "Co-authored-by";

pub(super) fn co_authors(commit: &JjCommit) -> Vec<CommitAuthor> {
    let author = commit_author(commit.author());
    let committer = commit_author(commit.committer());
    let timestamp = committer.timestamp_millis;
    let description = commit.description();
    // The trailer parser compiles a regex per call, too slow for every graph row.
    let has_trailer = description
        .as_bytes()
        .windows(CO_AUTHORED_BY.len())
        .any(|window| window.eq_ignore_ascii_case(CO_AUTHORED_BY.as_bytes()));
    let trailers = if has_trailer {
        parse_description_trailers(description)
    } else {
        Vec::new()
    };
    let trailers = trailers
        .into_iter()
        .filter(|trailer| trailer.key.eq_ignore_ascii_case(CO_AUTHORED_BY))
        .filter_map(|trailer| parse_identity(&trailer.value, timestamp));
    let committer = (!github_alias::is_web_flow(&committer.email)).then_some(committer);

    let mut people: Vec<CommitAuthor> = Vec::new();
    for person in trailers.chain(committer) {
        if !person.is_same_person(&author) && !people.iter().any(|p| p.is_same_person(&person)) {
            people.push(person);
        }
    }
    people
}

pub(super) fn commit_author(signature: &Signature) -> CommitAuthor {
    CommitAuthor::new(
        signature.name.clone(),
        signature.email.clone(),
        signature.timestamp.timestamp.0,
    )
}

fn parse_identity(value: &str, timestamp_millis: i64) -> Option<CommitAuthor> {
    let (name, email) = match value
        .trim()
        .strip_suffix('>')
        .and_then(|v| v.rsplit_once('<'))
    {
        Some((name, email)) => (name.trim(), email.trim()),
        None => (value.trim(), ""),
    };
    (!name.is_empty() || !email.is_empty())
        .then(|| CommitAuthor::new(name, email, timestamp_millis))
}
