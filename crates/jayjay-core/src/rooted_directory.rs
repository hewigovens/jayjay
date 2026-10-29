use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use cap_fs_ext::OpenOptionsSyncExt;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::ambient_authority;
use cap_std::fs::{Dir, File, OpenOptions};

use crate::filesystem::{io_error, safe_relative_path};
use crate::{JayError, JayResult};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Removal {
    RegularFile,
    Entry,
    Tree,
}

pub(crate) struct RootedDirectory {
    directory: Dir,
    root: PathBuf,
}

impl RootedDirectory {
    pub(crate) fn open(root: &Path) -> JayResult<Self> {
        let directory = Dir::open_ambient_dir(root, ambient_authority())
            .map_err(|error| io_error("open directory", root, error))?;
        Ok(Self {
            directory,
            root: root.to_owned(),
        })
    }

    pub(crate) fn open_regular_append(&self, relative: &Path) -> JayResult<File> {
        let relative = safe_relative_path(relative, "repository")?;
        let path = self.root.join(&relative);
        let mut options = OpenOptions::new();
        options
            .read(true)
            .append(true)
            .create(true)
            .follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.nonblock(true);
        let file = self
            .directory
            .open_with(&relative, &options)
            .map_err(|error| io_error("open regular file", &path, error))?;
        if !file
            .metadata()
            .map_err(|error| io_error("inspect", &path, error))?
            .is_file()
        {
            return Err(JayError::internal(format!(
                "not a regular file: {}",
                path.display()
            )));
        }
        Ok(file)
    }

    pub(crate) fn remove(&self, relative: &Path, mode: Removal) -> JayResult<()> {
        let relative = safe_relative_path(relative, "deletion")?;
        let path = self.root.join(&relative);
        let metadata = match self.directory.symlink_metadata(&relative) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) =>
            {
                return Ok(());
            }
            Err(error) => return Err(io_error("inspect deletion path", &path, error)),
        };
        if mode == Removal::RegularFile && !metadata.is_file() {
            return Ok(());
        }
        let result = if metadata.is_dir() && !metadata.file_type().is_symlink() {
            if mode == Removal::Tree {
                self.directory.remove_dir_all(&relative)
            } else {
                self.directory.remove_dir(&relative)
            }
        } else {
            self.directory.remove_file_or_symlink(&relative)
        };
        result.map_err(|error| io_error("remove", &path, error))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    use std::fs;
    use std::os::windows::fs::symlink_dir;

    #[test]
    fn deletion_unlinks_directory_symlinks_without_touching_the_target() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        fs::write(outside.path().join("keep"), "outside").unwrap();
        symlink_dir(outside.path(), root.path().join("link")).unwrap();
        RootedDirectory::open(root.path())
            .unwrap()
            .remove(Path::new("link"), Removal::Tree)
            .unwrap();
        assert!(fs::symlink_metadata(root.path().join("link")).is_err());
        assert_eq!(
            fs::read_to_string(outside.path().join("keep")).unwrap(),
            "outside"
        );
    }
}
