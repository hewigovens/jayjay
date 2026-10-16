use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use super::command_process::SyncToken;
use super::environment;
use crate::types::*;

const HTTPS_AUTH_HINT: &str =
    "Authentication failed. Use an SSH URL, or set up a Git credential helper for HTTPS.";
const SSH_AUTH_HINT: &str = "SSH authentication failed. Check that your key is loaded in ssh-agent and the host is in known_hosts.";

/// A `jj git clone` of `source` into its own folder, `destination`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloneRequest {
    pub source: String,
    pub destination: PathBuf,
}

impl CloneRequest {
    pub fn new(source: &str, location: &str) -> JayResult<Self> {
        let source = source.trim();
        if source.is_empty() {
            return Err(JayError::internal("Enter a repository URL."));
        }
        let destination = jj_lib::file_util::expand_home_path(
            location.trim(),
            environment::home_dir().as_deref(),
        );
        // jj runs from the parent folder, where a relative path would resolve a second time.
        if !destination.is_absolute() {
            return Err(JayError::internal("Enter a full path for the location."));
        }
        // A trailing `..` or a bare root would point the clone and its cleanup at another folder.
        let (Some(Component::Normal(name)), Some(parent)) =
            (destination.components().next_back(), destination.parent())
        else {
            return Err(JayError::internal("Enter a name for the clone."));
        };
        // Resolved once, so retargeting a symlinked parent mid-clone cannot redirect the cleanup.
        let parent = dunce::canonicalize(parent)
            .ok()
            .filter(|parent| parent.is_dir())
            .ok_or_else(|| JayError::internal(format!("{} is not a folder.", parent.display())))?;
        Ok(Self {
            source: source.to_owned(),
            destination: parent.join(name),
        })
    }

    /// The location text for a clone named `name` inside `folder`.
    pub fn location(folder: &str, name: &str) -> String {
        let name = name.trim();
        if name.is_empty() || folder.is_empty() {
            return format!("{folder}{name}");
        }
        let folder = folder.trim_end_matches(std::path::is_separator);
        format!("{folder}{}{name}", std::path::MAIN_SEPARATOR)
    }

    /// Splits typed location text at its last separator into the folder and the clone's name.
    pub fn split_location(location: &str) -> (&str, &str) {
        match location.rfind(std::path::is_separator) {
            Some(0) => location.split_at(1),
            Some(index) => (&location[..index], &location[index + 1..]),
            None => ("", location),
        }
    }

    /// The folder name `jj git clone` would pick: the source's last path component without `.git`.
    pub fn directory_name(source: &str) -> String {
        let source = source.trim().trim_end_matches(['/', '\\']);
        let last = source.rsplit(['/', '\\', ':']).next().unwrap_or_default();
        last.strip_suffix(".git").unwrap_or(last).to_owned()
    }

    /// Repositories usually live side by side, so a new clone defaults to the folder holding the most recent one.
    pub fn default_location(recent_repositories: &[String]) -> PathBuf {
        recent_repositories
            .iter()
            .filter_map(|path| Path::new(path).parent())
            .find(|parent| parent.is_dir())
            .map(Path::to_path_buf)
            .or_else(environment::home_dir)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Clones into `destination`, which must be absent or an empty folder; on failure or cancel it is put back that way.
    pub fn run(&self, sync: &SyncToken) -> JayResult<()> {
        // Cleanup would delete or replace a symlink, so even one to an empty folder counts as taken.
        let existed = match fs::symlink_metadata(&self.destination) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => false,
            Ok(metadata)
                if metadata.is_dir()
                    && fs::read_dir(&self.destination)
                        .is_ok_and(|mut entries| entries.next().is_none()) =>
            {
                true
            }
            _ => {
                return Err(JayError::internal(format!(
                    "{} already exists.",
                    self.destination.display()
                )));
            }
        };
        let jj = environment::require_jj()?;
        let mut command = environment::command(jj);
        command
            .current_dir(self.destination.parent().unwrap_or(Path::new(".")))
            // No terminal can answer a credential prompt, so Git fails instead of waiting on one.
            .env("GIT_TERMINAL_PROMPT", "0")
            .args(["git", "clone", "--", &self.source])
            .arg(&self.destination);
        let result = sync
            .output(&mut command, "jj git clone")
            .and_then(|output| {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(JayError::internal(failure_message(stderr.trim())))
            });
        if result.is_err() {
            self.discard_partial_clone(existed);
        }
        result
    }

    /// jj cleans up after a failed clone itself, but a canceled one leaves its partial `.jj` and `.git` behind.
    fn discard_partial_clone(&self, existed: bool) {
        let _ = fs::remove_dir_all(&self.destination);
        if existed {
            let _ = fs::create_dir(&self.destination);
        }
    }
}

fn failure_message(stderr: &str) -> String {
    let mentions = |needles: &[&str]| needles.iter().any(|needle| stderr.contains(needle));
    let hint = if mentions(&[
        "terminal prompts disabled",
        "could not read Username",
        "Authentication failed",
    ]) {
        Some(HTTPS_AUTH_HINT)
    } else if mentions(&[
        "Permission denied (publickey",
        "Host key verification failed",
    ]) {
        Some(SSH_AUTH_HINT)
    } else {
        None
    };
    match hint {
        Some(hint) => format!("{hint}\n\n{stderr}"),
        None if stderr.is_empty() => "jj git clone failed".to_owned(),
        None => stderr.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn directory_name_follows_the_last_source_component() {
        for (source, name) in [
            ("https://github.com/jj-vcs/jj.git", "jj"),
            ("https://github.com/jj-vcs/jj/", "jj"),
            ("git@github.com:jj-vcs/jj.git", "jj"),
            ("git@host:repo.git", "repo"),
            ("ssh://git@host:22/team/tool", "tool"),
            (r"C:\src\project", "project"),
            ("  ", ""),
        ] {
            assert_eq!(CloneRequest::directory_name(source), name, "{source}");
        }
    }

    #[test]
    fn new_takes_an_absolute_location_ending_in_the_clone_folder() {
        let folder = tempfile::tempdir().expect("tempdir");
        let folder = folder.path().to_str().expect("utf-8 path");
        for location in ["relative/dir", "/", &format!("{folder}/x/..")] {
            assert!(
                CloneRequest::new("https://host/x.git", location).is_err(),
                "{location:?}"
            );
        }
        assert!(CloneRequest::new("https://host/x.git", &format!("{folder}/missing/x")).is_err());
        let location = CloneRequest::location(folder, "x");
        let request =
            CloneRequest::new(" https://host/x.git ", &format!(" {location} ")).expect("valid");
        assert_eq!(request.source, "https://host/x.git");
        assert_eq!(
            request.destination,
            dunce::canonicalize(folder).expect("canonical").join("x")
        );
    }

    #[test]
    fn location_and_split_location_round_trip_folder_and_name() {
        let sep = std::path::MAIN_SEPARATOR;
        assert_eq!(
            CloneRequest::location("~/src", "jj"),
            format!("~/src{sep}jj")
        );
        assert_eq!(
            CloneRequest::location(&format!("~/src{sep}"), "jj"),
            format!("~/src{sep}jj")
        );
        assert_eq!(CloneRequest::location("~/src", " "), "~/src");
        assert_eq!(CloneRequest::location("", "jj"), "jj");
        assert_eq!(
            CloneRequest::location(&sep.to_string(), "jj"),
            format!("{sep}jj")
        );
        assert_eq!(
            CloneRequest::split_location(&format!("~/src{sep}jj")),
            ("~/src", "jj")
        );
        assert_eq!(
            CloneRequest::split_location(&format!("~/src{sep}")),
            ("~/src", "")
        );
        assert_eq!(
            CloneRequest::split_location(&format!("{sep}jj")),
            (&*sep.to_string(), "jj")
        );
        assert_eq!(CloneRequest::split_location("jj"), ("", "jj"));
    }

    #[test]
    fn failure_message_leads_with_how_to_fix_authentication() {
        let https =
            "fatal: could not read Username for 'https://github.com': terminal prompts disabled";
        let ssh = "git@github.com: Permission denied (publickey).";
        assert!(failure_message(https).starts_with(HTTPS_AUTH_HINT));
        assert!(failure_message(ssh).starts_with(SSH_AUTH_HINT));
        assert_eq!(
            failure_message("fatal: repository not found"),
            "fatal: repository not found"
        );
    }

    #[test]
    fn default_location_is_the_parent_of_the_first_recent_repository_that_still_exists() {
        let root = tempfile::tempdir().expect("tempdir");
        let recent = vec![
            "/nonexistent-jayjay-parent/repo".to_owned(),
            root.path().join("repo").display().to_string(),
        ];
        assert_eq!(CloneRequest::default_location(&recent), root.path());
    }
}
