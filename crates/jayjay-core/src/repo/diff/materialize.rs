use jj_lib::conflicts::MaterializedTreeValue;
use jj_lib::object_id::ObjectId;

use crate::repo::support::block_on_result;
use crate::types::*;

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
