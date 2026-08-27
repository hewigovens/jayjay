//! On-disk avatar cache and the blocking fetch that fills it.

use std::path::PathBuf;

use directories::ProjectDirs;
use md5::{Digest, Md5};

use super::resolve::{AvatarSource, avatar_source, bot_avatar_url, gitlab_avatar_url};

const AVATAR_BYTE_CAP: u32 = 2 * 1024 * 1024; // hard cap 2MB

pub(super) fn email_md5(email: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(email.trim().to_lowercase().as_bytes());
    hex::encode(hasher.finalize())
}

pub fn cache_path(email: &str) -> Option<PathBuf> {
    ProjectDirs::from("dev", "hewig", "jayjay").map(|dirs| {
        dirs.cache_dir()
            .join("avatars")
            .join(format!("{}.png", email_md5(email)))
    })
}

/// Blocking fetch (run on a background executor); writes the PNG to cache on success.
pub fn fetch_blocking(email: &str) -> bool {
    let Some(path) = cache_path(email) else {
        return false;
    };
    if path.exists() {
        return true;
    }
    let url = match avatar_source(email) {
        Some(AvatarSource::Url(url)) => url,
        Some(AvatarSource::GitHubBot(id)) => match bot_avatar_url(&id) {
            Some(url) => url,
            None => return false,
        },
        Some(AvatarSource::GitLabUser(name)) => match gitlab_avatar_url(&name) {
            Some(url) => url,
            None => return false,
        },
        None => return false,
    };

    let Ok(bytes) = jayjay_network::HttpClient::default().get_bytes(
        &url,
        AVATAR_BYTE_CAP,
        &jayjay_network::Auth::default(),
    ) else {
        return false;
    };
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    std::fs::write(&path, &bytes).is_ok()
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;

    #[test]
    fn macos_cache_path_is_shared_with_the_swiftui_shell() {
        const EMAIL: &str = "Person@example.com";
        let home = directories::BaseDirs::new()
            .unwrap()
            .home_dir()
            .to_path_buf();

        assert_eq!(
            cache_path(EMAIL).unwrap(),
            home.join("Library")
                .join("Caches")
                .join("dev.hewig.jayjay")
                .join("avatars")
                .join(format!("{}.png", email_md5(EMAIL)))
        );
    }
}
