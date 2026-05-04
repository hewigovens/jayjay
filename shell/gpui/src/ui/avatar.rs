//! Author avatar fetcher / cache (GPUI shell only).
//!
//! Strategy mirrors `shell/mac/Sources/JayJay/Shared/GitHubAvatar.swift`:
//!   1. If email is `<id>+<user>@users.noreply.github.com`, try
//!      `https://github.com/<user>.png?size=N` (preserves account avatar).
//!   2. Fallback to Gravatar: `https://gravatar.com/avatar/<md5>?s=N&d=retro`.
//!
//! Disk cache: `$HOME/.cache/jayjay/avatars/<email-hash>.png`. Once on disk,
//! the GPUI `img(path)` element renders it directly from the file without any
//! network call.

use std::fmt::Write as _;
use std::io::Read;
use std::path::PathBuf;

use md5::{Digest, Md5};

const PIXEL_SIZE: u32 = 96; // 2x for ~24pt slot

pub fn email_md5(email: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(email.trim().to_lowercase().as_bytes());
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        let _ = write!(&mut hex, "{:02x}", byte);
    }
    hex
}

pub fn cache_path(email: &str) -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    let cache_root = dirs
        .home_dir()
        .join(".cache")
        .join("jayjay")
        .join("avatars");
    Some(cache_root.join(format!("{}.png", email_md5(email))))
}

fn url_for_email(email: &str) -> Option<String> {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return None;
    }
    if let Some(local) = trimmed.strip_suffix("@users.noreply.github.com") {
        let user = local.rsplit('+').next().unwrap_or(local);
        if !user.is_empty() {
            return Some(format!("https://github.com/{user}.png?size={PIXEL_SIZE}"));
        }
    }
    Some(format!(
        "https://gravatar.com/avatar/{}?s={PIXEL_SIZE}&d=retro",
        email_md5(email)
    ))
}

/// Blocking fetch — meant to run on a background executor.
/// Writes the PNG bytes to the cache path on success.
pub fn fetch_blocking(email: &str) -> bool {
    let Some(path) = cache_path(email) else {
        return false;
    };
    if path.exists() {
        return true;
    }
    let Some(url) = url_for_email(email) else {
        return false;
    };

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(std::time::Duration::from_secs(8)))
        .build()
        .into();
    let mut response = match agent.get(&url).call() {
        Ok(r) => r,
        Err(_) => return false,
    };
    if response.status().as_u16() != 200 {
        return false;
    }
    let mut bytes = Vec::with_capacity(8 * 1024);
    if response
        .body_mut()
        .as_reader()
        .take(2 * 1024 * 1024) // hard cap 2MB
        .read_to_end(&mut bytes)
        .is_err()
    {
        return false;
    }
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    std::fs::write(&path, &bytes).is_ok()
}

/// Stable index into `INITIAL_PALETTE` derived from the email — same email
/// always gets the same fallback color.
pub fn initial_color(email: &str) -> u32 {
    const PALETTE: &[u32] = &[
        0x4a5568, 0x6b46c1, 0x2563eb, 0x059669, 0xd97706, 0xdc2626, 0xdb2777, 0x0891b2,
    ];
    let h = email_md5(email);
    let byte = u8::from_str_radix(&h[..2], 16).unwrap_or(0) as usize;
    PALETTE[byte % PALETTE.len()]
}

pub fn initial(name: &str) -> char {
    name.chars()
        .find(|c| c.is_alphanumeric())
        .map(|c| c.to_uppercase().next().unwrap_or('?'))
        .unwrap_or('?')
}
