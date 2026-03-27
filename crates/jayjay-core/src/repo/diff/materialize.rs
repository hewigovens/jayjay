use jj_lib::conflicts::MaterializedTreeValue;
use jj_lib::object_id::ObjectId;

use crate::repo::support::block_on_result;
use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GitLfsPointerInfo {
    pub(super) oid: String,
    pub(super) size: usize,
}

pub(super) fn materialized_to_string(
    path: &jj_lib::repo_path::RepoPath,
    value: MaterializedTreeValue,
) -> CoreResult<Option<String>> {
    match value {
        MaterializedTreeValue::Absent => Ok(None),
        MaterializedTreeValue::AccessDenied(err) => Ok(Some(format!("<access denied: {err}>"))),
        MaterializedTreeValue::File(mut file) => {
            let read = file.read_all(path);
            let bytes = block_on_result(
                &format!("read file {}", path.as_internal_file_string()),
                read,
            )?;
            if bytes.contains(&0) {
                return Ok(Some(format!("<binary file ({} bytes)>", bytes.len())));
            }
            match String::from_utf8(bytes) {
                Ok(text) => Ok(Some(text)),
                Err(err) => Ok(Some(format!(
                    "<binary file ({} bytes)>",
                    err.into_bytes().len()
                ))),
            }
        }
        MaterializedTreeValue::Symlink { target, .. } => Ok(Some(format!("symlink -> {target}"))),
        MaterializedTreeValue::FileConflict(_) => Ok(Some("<conflicted file>".to_owned())),
        MaterializedTreeValue::OtherConflict { .. } => Ok(Some("<conflict>".to_owned())),
        MaterializedTreeValue::GitSubmodule(id) => {
            Ok(Some(format!("<git submodule {}>", id.hex())))
        }
        MaterializedTreeValue::Tree(_) => Ok(Some("<directory>".to_owned())),
    }
}

pub(super) fn parse_git_lfs_pointer(text: &str) -> Option<GitLfsPointerInfo> {
    let mut lines = text.lines();
    if lines.next()? != "version https://git-lfs.github.com/spec/v1" {
        return None;
    }

    let mut oid = None;
    let mut size = None;
    for line in lines {
        if let Some(value) = line.strip_prefix("oid sha256:") {
            oid = Some(value.to_owned());
        } else if let Some(value) = line.strip_prefix("size ") {
            size = value.parse::<usize>().ok();
        }
    }

    Some(GitLfsPointerInfo {
        oid: oid?,
        size: size?,
    })
}

pub(super) fn parse_binary_placeholder_size(text: &str) -> Option<usize> {
    text.strip_prefix("<binary file (")?
        .strip_suffix(" bytes)>")?
        .parse::<usize>()
        .ok()
}

pub(super) fn git_lfs_pointer_placeholder(pointer: &GitLfsPointerInfo) -> String {
    format!(
        "<git lfs pointer sha256:{} ({} bytes)>",
        short_oid(&pointer.oid),
        pointer.size
    )
}

pub(super) fn git_lfs_object_placeholder(pointer: &GitLfsPointerInfo) -> String {
    format!(
        "<git lfs object sha256:{} ({} bytes)>",
        short_oid(&pointer.oid),
        pointer.size
    )
}

fn short_oid(oid: &str) -> &str {
    oid.get(..12).unwrap_or(oid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_lfs_pointer_text() {
        let pointer = parse_git_lfs_pointer(
            "version https://git-lfs.github.com/spec/v1\n\
             oid sha256:496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4\n\
             size 742800\n",
        )
        .unwrap();

        assert_eq!(pointer.size, 742800);
        assert_eq!(
            pointer.oid,
            "496634778d7b9bdbdb4b98b43a08a00ce8d794ed135a0cb1f345bf6febc5b9b4"
        );
        assert_eq!(
            git_lfs_pointer_placeholder(&pointer),
            "<git lfs pointer sha256:496634778d7b (742800 bytes)>"
        );
    }

    #[test]
    fn parses_binary_placeholder_size() {
        assert_eq!(
            parse_binary_placeholder_size("<binary file (742800 bytes)>"),
            Some(742800)
        );
    }
}
