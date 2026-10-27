use crate::{JayError, JayResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{BufRead, Write};

pub(crate) const PROTOCOL: u32 = 1;
pub(crate) const FRAME_LIMIT: usize = 96 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteFolder {
    pub name: String,
    pub path: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteDirectory {
    pub path: String,
    pub parent: Option<String>,
    pub folders: Vec<RemoteFolder>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Hello {
    pub protocol: u32,
    pub home: String,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct Request {
    pub id: u64,
    pub path: String,
    pub method: String,
    pub args: Vec<Value>,
}
pub(crate) use super::response::Response;

pub(crate) fn read_frame(reader: &mut impl BufRead) -> JayResult<Option<Vec<u8>>> {
    let mut bytes = Vec::new();
    loop {
        let chunk = reader.fill_buf().map_err(JayError::internal)?;
        if chunk.is_empty() {
            return if bytes.is_empty() {
                Ok(None)
            } else {
                Err(JayError::internal(
                    "SSH helper returned an incomplete frame.",
                ))
            };
        }
        let end = chunk
            .iter()
            .position(|b| *b == b'\n')
            .map(|n| n + 1)
            .unwrap_or(chunk.len());
        if bytes.len() + end > FRAME_LIMIT {
            return Err(JayError::internal("SSH response exceeded the size limit."));
        }
        bytes.extend_from_slice(&chunk[..end]);
        reader.consume(end);
        if bytes.last() == Some(&b'\n') {
            return Ok(Some(bytes));
        }
    }
}

pub(crate) fn write_frame(writer: &mut impl Write, value: &impl Serialize) -> JayResult<()> {
    let bytes = serde_json::to_vec(value).map_err(JayError::internal)?;
    if bytes.len() >= FRAME_LIMIT {
        return Err(JayError::refused("SSH message exceeded the size limit."));
    }
    writer
        .write_all(&bytes)
        .and_then(|()| writer.write_all(b"\n"))
        .and_then(|()| writer.flush())
        .map_err(JayError::internal)
}
