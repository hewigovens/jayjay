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

        let mut entries = Vec::new();
        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(4, '\t').collect();
            if parts.len() < 4 {
                continue;
            }
            entries.push(OpLogEntry {
                id: parts[0].to_string(),
                description: parts[1].to_string(),
                timestamp: parts[2].to_string(),
                is_current: parts[3].trim() == "true",
            });
        }
        Ok(entries)
    }

    /// Restore the repo to a given operation via `jj op restore`.
    pub fn op_restore(&self, op_id: &str) -> CoreResult<()> {
        self.run_jj_reload(&["op", "restore", op_id])
    }
}
