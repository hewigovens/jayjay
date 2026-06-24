use super::Repo;
use crate::types::*;

impl Repo {
    /// List the last 20 operations from `jj op log`.
    pub fn op_log(&self) -> CoreResult<Vec<OpLogEntry>> {
        let stdout = self.run_jj(&[
            "op",
            "log",
            "--limit",
            "20",
            "--no-graph",
            "--template",
            r#"self.id() ++ "\t" ++ self.description() ++ "\t" ++ self.time().start() ++ "\t" ++ self.current_operation() ++ "\n""#,
        ])?;

        let mut raw: Vec<(String, String, String, bool)> = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() < 4 {
                continue;
            }
            raw.push((
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
                parts[3].trim() == "true",
            ));
        }

        let ids: Vec<String> = raw.iter().map(|(id, ..)| id.clone()).collect();
        let entries = raw
            .into_iter()
            .map(|(id, description, timestamp, is_current)| {
                let short_len = unique_prefix_len(&id, &ids);
                OpLogEntry {
                    id: ShortId::new(id, short_len),
                    description,
                    timestamp,
                    is_current,
                }
            })
            .collect();
        Ok(entries)
    }

    /// Restore the repo to a given operation via `jj op restore`.
    pub fn op_restore(&self, op_id: &str) -> CoreResult<()> {
        self.run_jj_reload(&["op", "restore", op_id])
    }

    /// Description of the operation the repo is currently at, read in-process from the loaded repo (no subprocess) so the status bar can show it cheaply.
    pub fn current_operation_description(&self) -> String {
        self.get_repo()
            .operation()
            .metadata()
            .description
            .lines()
            .next()
            .unwrap_or("")
            .to_owned()
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

#[cfg(test)]
mod tests {
    use super::unique_prefix_len;

    #[test]
    fn unique_prefix_grows_until_distinct() {
        let ids = vec!["abcd".to_owned(), "abce".to_owned(), "ffff".to_owned()];
        assert_eq!(unique_prefix_len("abcd", &ids), 4); // shares "abc" with abce
        assert_eq!(unique_prefix_len("abce", &ids), 4);
        assert_eq!(unique_prefix_len("ffff", &ids), 1); // unique at first char
    }
}
