/// Beta release builds set `JAYJAY_PRERELEASE=1`, since installed versions carry no `-beta` suffix.
pub fn is_prerelease_build() -> bool {
    option_env!("JAYJAY_PRERELEASE") == Some("1")
}
