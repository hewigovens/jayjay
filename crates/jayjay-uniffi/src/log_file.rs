use jayjay_core::LogFile;

#[uniffi::export]
fn install_log_file(version: String) {
    LogFile::install("swiftui", &version);
}

#[uniffi::export]
fn log_file_path() -> Option<String> {
    LogFile::path().map(|path| path.display().to_string())
}
