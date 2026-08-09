use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::filesystem::{io_error, is_executable, set_executable};
use crate::{CoreError, CoreResult};

pub(super) fn output_matches_selection(
    path: &Path,
    selected_text: &str,
    selected_exists: bool,
    selected_executable: Option<bool>,
) -> CoreResult<bool> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(!selected_exists);
        }
        Err(error) => return Err(io_error("inspect", path, error)),
    };
    if !selected_exists {
        return Ok(!metadata.is_file());
    }
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Ok(false);
    }
    if selected_executable.is_some_and(|executable| is_executable(&metadata) != executable) {
        return Ok(false);
    }
    fs::read(path)
        .map(|contents| contents == selected_text.as_bytes())
        .map_err(|error| io_error("read", path, error))
}

pub(super) fn copy_entry(
    source: &Path,
    right_root: &Path,
    relative: &Path,
    output: &Path,
    directory_output: bool,
) -> CoreResult<()> {
    let metadata =
        fs::symlink_metadata(source).map_err(|error| io_error("inspect", source, error))?;
    if !metadata.file_type().is_symlink() && !metadata.is_file() {
        return Err(CoreError::internal(format!(
            "cannot restore unsupported external diff entry: {}",
            source.display()
        )));
    }
    if directory_output {
        prepare_parent_directories(right_root, relative)?;
        remove_output_path(&right_root.join(relative))?;
    } else {
        remove_output_path(output)?;
    }
    if metadata.file_type().is_symlink() {
        let target =
            fs::read_link(source).map_err(|error| io_error("read symlink", source, error))?;
        create_symlink(source, &target, output)?;
        return Ok(());
    }
    fs::copy(source, output).map_err(|error| io_error("copy", output, error))?;
    set_executable(output, is_executable(&metadata))
}

pub(super) fn write_text(
    right_root: &Path,
    relative: &Path,
    output: &Path,
    directory_output: bool,
    text: &str,
    executable: Option<bool>,
) -> CoreResult<()> {
    if directory_output {
        prepare_write_path(right_root, relative)?;
    } else if fs::symlink_metadata(output).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err(CoreError::internal(format!(
            "refusing to write external diff through unsafe path: {}",
            output.display()
        )));
    }
    fs::write(output, text.as_bytes()).map_err(|error| io_error("write", output, error))?;
    if let Some(executable) = executable {
        set_executable(output, executable)?;
    }
    Ok(())
}

pub(super) fn remove_output_path(path: &Path) -> CoreResult<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(());
        }
        Err(error) => return Err(io_error("inspect", path, error)),
    };
    let result = if metadata.is_dir() && !metadata.file_type().is_symlink() {
        fs::remove_dir(path)
    } else {
        fs::remove_file(path)
    };
    result.map_err(|error| io_error("remove", path, error))
}

fn prepare_write_path(root: &Path, relative: &Path) -> CoreResult<()> {
    prepare_parent_directories(root, relative)?;
    let output = root.join(relative);
    match fs::symlink_metadata(&output) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err(CoreError::internal(format!(
                "refusing to write external diff through unsafe path: {}",
                output.display()
            )));
        }
        Ok(metadata) if metadata.is_dir() => {
            fs::remove_dir(&output)
                .map_err(|error| io_error("remove directory", &output, error))?;
        }
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(io_error("inspect", &output, error)),
    }
    Ok(())
}

fn prepare_parent_directories(root: &Path, relative: &Path) -> CoreResult<()> {
    let mut directory = root.to_owned();
    if let Some(parent) = relative.parent() {
        for component in parent.components() {
            directory.push(component);
            match fs::symlink_metadata(&directory) {
                Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
                Ok(_) => {
                    return Err(CoreError::internal(format!(
                        "refusing to write external diff through unsafe path: {}",
                        directory.display()
                    )));
                }
                Err(error) if error.kind() == ErrorKind::NotFound => {
                    fs::create_dir(&directory)
                        .map_err(|error| io_error("create directory", &directory, error))?;
                }
                Err(error) => return Err(io_error("inspect", &directory, error)),
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn create_symlink(_source: &Path, target: &Path, output: &Path) -> CoreResult<()> {
    std::os::unix::fs::symlink(target, output)
        .map_err(|error| io_error("create symlink", output, error))
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path, output: &Path) -> CoreResult<()> {
    let result = if source.is_dir() {
        std::os::windows::fs::symlink_dir(target, output)
    } else {
        std::os::windows::fs::symlink_file(target, output)
    };
    result.map_err(|error| io_error("create symlink", output, error))
}

#[cfg(not(any(unix, windows)))]
fn create_symlink(_source: &Path, _target: &Path, output: &Path) -> CoreResult<()> {
    Err(CoreError::internal(format!(
        "cannot create external diff symlink on this platform: {}",
        output.display()
    )))
}
