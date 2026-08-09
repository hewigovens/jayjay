use std::fs;
use std::path::{Component, Path, PathBuf};

use futures::AsyncReadExt as _;

use crate::{CoreError, CoreResult};

pub(crate) async fn read_to_limit(
    reader: impl futures::AsyncRead + Unpin,
    limit: usize,
) -> std::io::Result<(Vec<u8>, bool)> {
    let mut bytes = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)
        .await?;
    let truncated = bytes.len() > limit;
    if truncated {
        bytes.clear();
        bytes.shrink_to_fit();
    }
    Ok((bytes, truncated))
}

pub(crate) fn normalized_absolute_path(path: &str) -> String {
    let path = Path::new(path);
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized.to_string_lossy().into_owned()
}

pub(crate) fn safe_relative_path(path: &Path, kind: &str) -> CoreResult<PathBuf> {
    if path.as_os_str().is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(CoreError::internal(format!(
            "invalid {kind} path: {}",
            path.display()
        )));
    }
    Ok(path.to_owned())
}

pub(crate) fn io_error(action: &str, path: &Path, error: std::io::Error) -> CoreError {
    CoreError::internal(format!("{action} {}: {error}", path.display()))
}

#[cfg(unix)]
pub(crate) fn is_executable(metadata: &fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
pub(crate) fn is_executable(_: &fs::Metadata) -> bool {
    false
}

#[cfg(unix)]
pub(crate) fn set_executable(path: &Path, executable: bool) -> CoreResult<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let mut permissions = fs::metadata(path)
        .map_err(|error| io_error("read permissions", path, error))?
        .permissions();
    let mode = permissions.mode();
    let selected_mode = if executable {
        mode | 0o111
    } else {
        mode & !0o111
    };
    if selected_mode == mode {
        return Ok(());
    }
    permissions.set_mode(selected_mode);
    fs::set_permissions(path, permissions)
        .map_err(|error| io_error("write permissions", path, error))
}

#[cfg(not(unix))]
pub(crate) fn set_executable(_: &Path, _: bool) -> CoreResult<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::read_to_limit;

    fn read(data: Vec<u8>, limit: usize) -> (Vec<u8>, bool) {
        pollster::block_on(read_to_limit(futures::io::Cursor::new(data), limit)).unwrap()
    }

    #[test]
    fn caps_async_reads() {
        assert_eq!(read(vec![b'a'; 100], 1024), (vec![b'a'; 100], false));
        assert_eq!(read(vec![b'a'; 64], 64), (vec![b'a'; 64], false));
        assert_eq!(read(vec![b'a'; 65], 64), (Vec::new(), true));
    }
}
