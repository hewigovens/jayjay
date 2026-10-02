use serde::Deserialize;

use crate::repo::hosted_repo::RepoHost;

#[derive(Deserialize)]
pub(crate) struct GitLabProjectResponse {
    http_url_to_repo: String,
    ssh_url_to_repo: String,
    path_with_namespace: String,
}

impl GitLabProjectResponse {
    /// Pinned to gitlab.com: the URL comes from the API, and the generic remote-URL check accepts any host and local paths.
    pub(crate) fn pinned_clone_url(&self, ssh: bool) -> Option<&str> {
        let url = if ssh {
            &self.ssh_url_to_repo
        } else {
            &self.http_url_to_repo
        };
        RepoHost::GitLab.hosts(url).then_some(url)
    }

    pub(crate) fn namespace(&self) -> Option<&str> {
        self.path_with_namespace
            .rsplit_once('/')
            .map(|(namespace, _)| namespace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(http_url: &str, ssh_url: &str, namespace: &str) -> GitLabProjectResponse {
        serde_json::from_str(&format!(
            r#"{{"http_url_to_repo": "{http_url}", "ssh_url_to_repo": "{ssh_url}",
                "path_with_namespace": "{namespace}"}}"#
        ))
        .unwrap()
    }

    #[test]
    fn clone_url_follows_the_origin_scheme_and_stays_on_gitlab_com() {
        let fork = project(
            "https://gitlab.com/group/sub/fork.git",
            "git@gitlab.com:group/sub/fork.git",
            "group/sub/fork",
        );
        assert_eq!(
            fork.pinned_clone_url(false),
            Some("https://gitlab.com/group/sub/fork.git")
        );
        assert_eq!(
            fork.pinned_clone_url(true),
            Some("git@gitlab.com:group/sub/fork.git")
        );
        assert_eq!(fork.namespace(), Some("group/sub"));

        let off_host = project(
            "https://github.com/alice/fork.git",
            "ext::sh -c evil",
            "fork",
        );
        assert_eq!(off_host.pinned_clone_url(false), None);
        assert_eq!(off_host.pinned_clone_url(true), None);
        assert_eq!(off_host.namespace(), None);
    }
}
