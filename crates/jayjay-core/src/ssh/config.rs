use super::RemoteLocation;
use crate::json_store::JsonStore;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[derive(Clone, Default, Serialize, Deserialize)]
struct SavedMachines {
    /// Least recently used first.
    hosts: Vec<String>,
}

fn store() -> JsonStore<SavedMachines> {
    JsonStore::load(
        "ssh_machines",
        "JAYJAY_SSH_MACHINES_PATH",
        "ssh_machines.json",
        |_| {},
    )
}

pub fn remember_machine(machine: &str) {
    if RemoteLocation::new(machine, "~").is_ok() {
        store().update(|saved| {
            if saved.hosts.last().is_some_and(|host| host == machine) {
                return false;
            }
            saved.hosts.retain(|host| host != machine);
            saved.hosts.push(machine.into());
            true
        });
    }
}

pub fn last_machine() -> Option<String> {
    store().read().hosts.last().cloned()
}

pub fn machines() -> Vec<String> {
    let mut hosts: BTreeSet<String> = store().read().hosts.clone().into_iter().collect();
    if let Some(home) = crate::home_dir() {
        read_hosts(
            &home.join(".ssh/config"),
            &home,
            &mut BTreeSet::new(),
            &mut hosts,
        );
    }
    hosts.into_iter().collect()
}

fn read_hosts(
    path: &Path,
    home: &Path,
    visited: &mut BTreeSet<PathBuf>,
    hosts: &mut BTreeSet<String>,
) {
    if visited.len() >= 64 || !visited.insert(path.to_owned()) {
        return;
    }
    let Ok(metadata) = std::fs::metadata(path) else {
        return;
    };
    if metadata.len() > 1024 * 1024 {
        return;
    }
    let Ok(content) = std::fs::read_to_string(path) else {
        return;
    };
    for line in content.lines() {
        let line = line.trim().replace('=', " ");
        let Ok(words) = shell_words::split(&line) else {
            continue;
        };
        let Some((directive, values)) = words.split_first() else {
            continue;
        };
        if directive.eq_ignore_ascii_case("host") {
            hosts.extend(
                values
                    .iter()
                    .take_while(|v| !v.starts_with('#'))
                    .filter(|v| !v.contains(['*', '?', '!']) && RemoteLocation::new(v, "~").is_ok())
                    .cloned(),
            );
        } else if directive.eq_ignore_ascii_case("include") {
            for value in values.iter().take_while(|v| !v.starts_with('#')) {
                let pattern = if let Some(rest) = value.strip_prefix("~/") {
                    home.join(rest)
                } else if Path::new(value).is_absolute() {
                    value.into()
                } else {
                    home.join(".ssh").join(value)
                };
                if let Ok(paths) = glob::glob(&pattern.to_string_lossy()) {
                    for included in paths.flatten() {
                        read_hosts(&included, home, visited, hosts);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn suggests_literal_hosts_from_includes_without_resolving_match_rules() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir(temp.path().join(".ssh")).unwrap();
        std::fs::write(temp.path().join(".ssh/config"), "Host DevBox prod # comment\nHost *.internal !excluded\nInclude hosts-*\nMatch host prod\n User alice\n").unwrap();
        std::fs::write(
            temp.path().join(".ssh/hosts-a"),
            "Host linux\nInclude config\n",
        )
        .unwrap();
        let mut hosts = BTreeSet::new();
        read_hosts(
            &temp.path().join(".ssh/config"),
            temp.path(),
            &mut BTreeSet::new(),
            &mut hosts,
        );
        assert_eq!(
            hosts.into_iter().collect::<Vec<_>>(),
            ["DevBox", "linux", "prod"]
        );
    }
}
