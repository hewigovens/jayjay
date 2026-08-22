use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::normalize_repository_path;
use crate::repo::workspace_primary_root;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoGroup {
    pub path: String,
    pub workspaces: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoListGroups {
    pub pinned: Vec<RepoGroup>,
    pub recent: Vec<RepoGroup>,
}

/// Pinned entries stay top-level. Reads the filesystem; call off the UI thread.
pub fn group_repositories(pinned: &[String], recents: &[String]) -> RepoListGroups {
    group_with(pinned, recents, workspace_primary_root)
}

fn group_with(
    pinned: &[String],
    recents: &[String],
    primary_root: impl Fn(&str) -> Option<String>,
) -> RepoListGroups {
    // Later writers win: a pinned entry owns the group.
    let mut owner_by_canonical: HashMap<String, &String> = HashMap::new();
    for path in recents.iter().chain(pinned) {
        owner_by_canonical.insert(canonical(path), path);
    }
    let mut workspaces_by_owner: HashMap<&String, Vec<String>> = HashMap::new();
    let mut hidden: HashSet<&String> = HashSet::new();
    for path in recents {
        let own = canonical(path);
        // Same spelling can appear in both lists, so ownership is by entry, not by value.
        if !std::ptr::eq(owner_by_canonical[&own], path) {
            hidden.insert(path);
            continue;
        }
        let Some(owner) = primary_root(path)
            .map(|root| canonical(&root))
            .filter(|root| *root != own)
            .and_then(|root| owner_by_canonical.get(&root))
        else {
            continue;
        };
        workspaces_by_owner
            .entry(owner)
            .or_default()
            .push(path.clone());
        hidden.insert(path);
    }
    let group = |path: &String| {
        let mut workspaces = workspaces_by_owner.get(path).cloned().unwrap_or_default();
        workspaces.sort_by_cached_key(|path| display_name(path));
        RepoGroup {
            path: path.clone(),
            workspaces,
        }
    };
    RepoListGroups {
        pinned: pinned.iter().map(group).collect(),
        recent: recents
            .iter()
            .filter(|path| !hidden.contains(path))
            .map(group)
            .collect(),
    }
}

fn canonical(path: &str) -> String {
    normalize_repository_path(Path::new(path))
        .components()
        .collect::<std::path::PathBuf>()
        .to_string_lossy()
        .into_owned()
}

fn display_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::group_with;

    fn primary_root(path: &str) -> Option<String> {
        let root = match path {
            "/work/main" | "/work/agent-a" | "/work/agent-b" => "/work/main",
            "/work/orphan-ws" => "/gone/main",
            "/work/other" => "/work/other",
            _ => return None,
        };
        Some(root.to_owned())
    }

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn paths(groups: &[super::RepoGroup]) -> Vec<&str> {
        groups.iter().map(|group| group.path.as_str()).collect()
    }

    #[test]
    fn recent_workspaces_nest_under_their_listed_root() {
        let groups = group_with(
            &strings(&["/work/main"]),
            &strings(&[
                "/work/agent-b",
                "/work/other",
                "/work/orphan-ws",
                "/not-a-repo",
                "/work/agent-a",
            ]),
            primary_root,
        );
        assert_eq!(paths(&groups.pinned), ["/work/main"]);
        assert_eq!(
            groups.pinned[0].workspaces,
            ["/work/agent-a", "/work/agent-b"]
        );
        assert_eq!(
            paths(&groups.recent),
            ["/work/other", "/work/orphan-ws", "/not-a-repo"]
        );
        assert!(
            groups
                .recent
                .iter()
                .all(|group| group.workspaces.is_empty())
        );
    }

    #[test]
    fn primary_root_spelling_does_not_have_to_match_the_listed_entry() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path().join("main");
        let workspace = temp.path().join("agent");
        std::fs::create_dir_all(&root).expect("root");
        std::fs::create_dir_all(&workspace).expect("workspace");
        let listed = |path: &std::path::Path| super::canonical(&path.to_string_lossy());
        let unresolved_root = root.join(".").to_string_lossy().into_owned();

        let groups = group_with(&[listed(&root)], &[listed(&workspace)], |_| {
            Some(unresolved_root.clone())
        });

        assert_eq!(groups.pinned[0].workspaces, [listed(&workspace)]);
        assert!(groups.recent.is_empty(), "{:?}", groups.recent);
    }

    #[test]
    fn pinned_entries_stay_top_level_and_own_their_duplicates() {
        let groups = group_with(
            &strings(&["/work/agent-a", "/work/main"]),
            &strings(&["/work/main", "/work/agent-b", "/work/main/"]),
            primary_root,
        );
        assert_eq!(paths(&groups.pinned), ["/work/agent-a", "/work/main"]);
        assert!(groups.pinned[0].workspaces.is_empty());
        assert_eq!(groups.pinned[1].workspaces, ["/work/agent-b"]);
        assert!(groups.recent.is_empty(), "{:?}", groups.recent);
    }
}
