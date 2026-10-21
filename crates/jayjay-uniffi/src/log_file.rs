use jayjay_core::LogFile;

#[uniffi::export]
fn install_log_file(version: String) {
    LogFile::install("swiftui", &version);
}

#[uniffi::export]
fn set_debug_logging(enabled: bool) {
    LogFile::set_debug(enabled);
}

#[uniffi::export]
fn is_prerelease_build() -> bool {
    jayjay_core::is_prerelease_build()
}

#[uniffi::export]
fn log_file_path() -> Option<String> {
    LogFile::path().map(|path| path.display().to_string())
}
