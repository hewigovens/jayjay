use jayjay_core::{RepoHost, TokenSource};

#[uniffi::remote(Enum)]
pub enum RepoHost {
    GitHub,
    Codeberg,
    GitLab,
    Cursor,
}

#[uniffi::export]
fn token_hosts() -> Vec<RepoHost> {
    RepoHost::TOKEN_HOSTS.to_vec()
}

#[uniffi::export]
fn repo_host_name(host: RepoHost) -> String {
    host.display_name().to_owned()
}

#[uniffi::export]
fn host_token_source(host: RepoHost) -> Option<String> {
    host.token_source().map(TokenSource::label)
}
