use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

const FILE_NAME: &str = "jayjay-gpui.log";
const ROTATED_FILE_NAME: &str = "jayjay-gpui.1.log";
const MAX_BYTES: u64 = 2 * 1024 * 1024;

/// Mirrors log records to stderr and to a size-capped file, since the Windows GUI subsystem has no console.
pub struct LogFile {
    dir: PathBuf,
    file: Mutex<Option<File>>,
}

impl LogFile {
    pub fn install() {
        let Some(dir) = Self::dir() else { return };
        let logger = Self::new(dir);
        if log::set_boxed_logger(Box::new(logger)).is_ok() {
            log::set_max_level(LevelFilter::Info);
        }
    }

    fn dir() -> Option<PathBuf> {
        jayjay_core::AppDirs::new().map(|dirs| dirs.data.join("logs"))
    }

    fn new(dir: PathBuf) -> Self {
        let file = Mutex::new(Self::open(&dir));
        Self { dir, file }
    }

    fn open(dir: &Path) -> Option<File> {
        fs::create_dir_all(dir).ok()?;
        let path = dir.join(FILE_NAME);
        if path.metadata().is_ok_and(|meta| meta.len() > MAX_BYTES) {
            let _ = fs::rename(&path, dir.join(ROTATED_FILE_NAME));
        }
        OpenOptions::new().create(true).append(true).open(path).ok()
    }

    fn append(&self, line: &str) {
        let Ok(mut file) = self.file.lock() else {
            return;
        };
        if file
            .as_ref()
            .is_some_and(|file| file.metadata().is_ok_and(|meta| meta.len() > MAX_BYTES))
        {
            *file = None;
            *file = Self::open(&self.dir);
        }
        if let Some(file) = file.as_mut() {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

impl Log for LogFile {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // GPUI and its platform crates log chatty info lines; keep only their warnings.
        let ceiling = if metadata.target().starts_with("jayjay") {
            Level::Info
        } else {
            Level::Warn
        };
        metadata.level() <= ceiling
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let line = format!(
            "{} {:<5} {} {}\n",
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            record.level(),
            record.target(),
            record.args()
        );
        eprint!("{line}");
        self.append(&line);
    }

    fn flush(&self) {
        if let Ok(mut file) = self.file.lock()
            && let Some(file) = file.as_mut()
        {
            let _ = file.flush();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oversized_log_rotates() {
        let dir = tempfile::tempdir().unwrap();
        let log = LogFile::new(dir.path().to_owned());
        let oversized = "x".repeat(MAX_BYTES as usize + 1);
        log.append(&oversized);
        log.append("fresh\n");
        assert_eq!(
            fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
            "fresh\n"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join(ROTATED_FILE_NAME)).unwrap(),
            oversized
        );
    }
}
