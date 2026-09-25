use futures::{StreamExt as _, TryStreamExt as _};
use jj_lib::object_id::ObjectId as _;
use jj_lib::op_walk;
use jj_lib::operation::Operation;

use super::Repo;
use super::support::block_on_result;
use crate::types::*;

const OP_LOG_LIMIT: usize = 20;

impl Repo {
    /// The 20 most recent operations, newest first. Snapshots first so the current entry already holds pending edits and restoring to it later cannot drop them.
    pub fn op_log(&self) -> CoreResult<Vec<OpLogEntry>> {
        self.refresh_working_copy()?;
        let repo = self.get_repo();
        let ops: Vec<Operation> = block_on_result(
            "walk operations",
            op_walk::walk_ancestors(std::slice::from_ref(repo.operation()))
                .take(OP_LOG_LIMIT)
                .try_collect(),
        )?;
        let ids: Vec<String> = ops.iter().map(|op| op.id().hex()).collect();
        let entries = ops
            .iter()
            .zip(&ids)
            .map(|(op, id)| {
                let metadata = op.metadata();
                OpLogEntry {
                    id: ShortId::new(id.clone(), unique_prefix_len(id, &ids)),
                    description: shorten_embedded_ids(&metadata.description),
                    timestamp_millis: metadata.time.start.timestamp.0,
                    is_current: op.id() == repo.op_id(),
                }
            })
            .collect();
        Ok(entries)
    }

    /// Restore the repo to a given operation via `jj op restore`.
    pub fn op_restore(&self, op_id: &str) -> CoreResult<()> {
        let _write = self.write_guard()?;
        self.run_jj_reload(&["op", "restore", op_id])
    }

    /// Whether the loaded repo matches the sole on-disk operation head.
    pub fn is_at_operation_head(&self) -> CoreResult<bool> {
        let repo = self.get_repo();
        let heads = block_on_result("read operation heads", repo.op_heads_store().get_op_heads())?;
        Ok(heads.len() == 1 && heads[0] == *repo.op_id())
    }

    /// Description of the operation the repo is currently at, read in-process from the loaded repo (no subprocess) so the status bar can show it cheaply.
    pub fn current_operation_description(&self) -> String {
        shorten_embedded_ids(
            self.get_repo()
                .operation()
                .metadata()
                .description
                .lines()
                .next()
                .unwrap_or(""),
        )
    }
}

/// Shortest prefix of `id` that is unique among `all` (hex ids, so byte slicing is safe). Falls back to the full length when nothing distinguishes it.
fn unique_prefix_len(id: &str, all: &[String]) -> u32 {
    for len in 1..=id.len() {
        let prefix = &id[..len];
        if all.iter().filter(|other| other.starts_with(prefix)).count() == 1 {
            return len as u32;
        }
    }
    id.len() as u32
}

/// jj writes full 128-hex operation ids (and 40-hex commit ids) into descriptions such as "restore to operation …"; show 12 characters like `jj op log`.
fn shorten_embedded_ids(description: &str) -> String {
    const COMMIT_ID_LEN: usize = 40;
    const OPERATION_ID_LEN: usize = 128;
    const SHORT_LEN: usize = 12;
    description
        .split(' ')
        .map(|word| {
            if matches!(word.len(), COMMIT_ID_LEN | OPERATION_ID_LEN)
                && word.bytes().all(|b| b.is_ascii_hexdigit())
            {
                &word[..SHORT_LEN]
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{shorten_embedded_ids, unique_prefix_len};

    #[test]
    fn long_hex_ids_in_descriptions_are_shortened() {
        let op = "909d5af891b700efb60cf5c1469d8d4a3b6557a4cc0d0a8382cc5e413e8a97d4d6149cd61939704e0300b9d5468a2d386a0f5687211facf2bd693eaf61529b5f";
        assert_eq!(
            shorten_embedded_ids(&format!("restore to operation {op}")),
            "restore to operation 909d5af891b7"
        );
        assert_eq!(
            shorten_embedded_ids("rebase commit 55e50b53f4f09d24c56bc5c7d90b9545c6b871f3"),
            "rebase commit 55e50b53f4f0"
        );
        let remote = "0123456789abcdef0123456789abcdef";
        assert_eq!(
            shorten_embedded_ids(&format!("add git remote {remote}")),
            format!("add git remote {remote}")
        );
    }

    #[test]
    fn unique_prefix_grows_until_distinct() {
        let ids = vec!["abcd".to_owned(), "abce".to_owned(), "ffff".to_owned()];
        assert_eq!(unique_prefix_len("abcd", &ids), 4); // shares "abc" with abce
        assert_eq!(unique_prefix_len("abce", &ids), 4);
        assert_eq!(unique_prefix_len("ffff", &ids), 1); // unique at first char
    }
}
