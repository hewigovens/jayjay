use jayjay_core::github_alias;

#[uniffi::export]
pub fn github_alias_noreply_email(email: String) -> Option<String> {
    github_alias::noreply_email(&email)
}
