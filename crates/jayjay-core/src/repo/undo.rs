use super::Repo;
use crate::types::*;

impl Repo {
    /// List the last 20 operations from `jj op log`.
    pub fn op_log(&self) -> CoreResult<Vec<OpLogEntry>> {
        let output = std::process::Command::new(&super::jj_binary())
            .current_dir(&self.path)
            .args([
                "op",
                "log",
                "--limit",
                "20",
                "--no-graph",
                "--template",
                r#"self.id() ++ "\t" ++ self.description() ++ "\t" ++ self.time().start() ++ "\t" ++ self.current_operation() ++ "\n""#,
            ])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("run jj op log: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("jj op log failed: {stderr}"),
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
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
        let output = std::process::Command::new(&super::jj_binary())
            .current_dir(&self.path)
            .args(["op", "restore", op_id])
            .output()
            .map_err(|e| CoreError::Internal {
                message: format!("run jj op restore: {e}"),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(CoreError::Internal {
                message: format!("jj op restore failed: {stderr}"),
            });
        }
        self.reload()
    }
}
