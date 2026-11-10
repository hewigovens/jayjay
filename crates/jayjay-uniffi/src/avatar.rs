use jayjay_core::avatar::{self, AvatarSource};

#[uniffi::remote(Enum)]
pub enum AvatarSource {
    Url(String),
    GitHubBot(String),
    GitLabUser(String),
}

#[uniffi::export]
fn avatar_cache_key(email: &str) -> String {
    avatar::cache_key(email)
}

#[uniffi::export]
fn avatar_source(email: &str, pixel_size: u32) -> Option<AvatarSource> {
    AvatarSource::for_email(email, pixel_size)
}

#[uniffi::export]
fn avatar_monogram_color(email: &str) -> u32 {
    avatar::monogram_color(email)
}
