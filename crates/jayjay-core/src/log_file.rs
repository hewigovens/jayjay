use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use log::{Level, LevelFilter, Log, Metadata, Record};

const FILE_NAME: &str = "jayjay.log";
const ROTATED_FILE_NAME: &str = "jayjay.1.log";
const MAX_BYTES: u64 = 2 * 1024 * 1024;

pub struct LogFile {
    dir: Option<PathBuf>,
    file: Mutex<Option<File>>,
}

impl LogFile {
    pub fn install(shell: &str, version: &str) {
        if log::set_boxed_logger(Box::new(Self::new(Self::dir()))).is_ok() {
            log::set_max_level(LevelFilter::Info);
        }
        log::info!(
            "launching {shell} {version} on {} {}",
            std::env::consts::OS,
            std::env::consts::ARCH
        );
    }

    pub fn set_debug(enabled: bool) {
        log::set_max_level(if enabled {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        });
    }

    pub fn path() -> Option<PathBuf> {
        Self::dir().map(|dir| dir.join(FILE_NAME))
    }

    fn dir() -> Option<PathBuf> {
        crate::AppDirs::new().map(|dirs| dirs.data.join("logs"))
    }

    fn new(dir: Option<PathBuf>) -> Self {
        let file = Mutex::new(dir.as_deref().and_then(Self::open));
        Self { dir, file }
    }

    fn open(dir: &Path) -> Option<File> {
        fs::create_dir_all(dir).ok()?;
        let path = dir.join(FILE_NAME);
        let oversized = path.metadata().is_ok_and(|meta| meta.len() > MAX_BYTES);
        let truncate = oversized && fs::rename(&path, dir.join(ROTATED_FILE_NAME)).is_err();
        let mut options = OpenOptions::new();
        if truncate {
            options.write(true).truncate(true);
        } else {
            options.append(true);
        }
        options.create(true).open(path).ok()
    }

    fn append(&self, line: &str) {
        let (Some(dir), Ok(mut file)) = (&self.dir, self.file.lock()) else {
            return;
        };
        if file
            .as_ref()
            .is_some_and(|file| file.metadata().is_ok_and(|meta| meta.len() > MAX_BYTES))
        {
            *file = None;
            *file = Self::open(dir);
        }
        if let Some(file) = file.as_mut() {
            let _ = file.write_all(line.as_bytes());
        }
    }
}

impl Log for LogFile {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let ceiling = if metadata.target().starts_with("jayjay") {
            log::max_level()
        } else {
            LevelFilter::Warn
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
        if record.level() <= Level::Warn {
            let _ = std::io::stderr().write_all(line.as_bytes());
        }
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
        let log = LogFile::new(Some(dir.path().to_owned()));
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

    #[test]
    fn oversized_log_truncates_when_rotation_fails() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(ROTATED_FILE_NAME).join("blocker")).unwrap();
        let log = LogFile::new(Some(dir.path().to_owned()));
        log.append(&"x".repeat(MAX_BYTES as usize + 1));
        log.append("fresh\n");
        assert_eq!(
            fs::read_to_string(dir.path().join(FILE_NAME)).unwrap(),
            "fresh\n"
        );
    }
}
