use super::Repo;
use crate::types::*;

impl Repo {
    /// List all workspaces for this repo.
    pub fn workspace_list(&self) -> CoreResult<Vec<WorkspaceInfo>> {
        let output = self.run_jj(&["workspace", "list"])?;
        let current_name = self.workspace_name.as_str();
        let mut workspaces = Vec::new();

        for line in output.lines() {
            let name = line.split(':').next().unwrap_or("").trim().to_owned();
            if name.is_empty() {
                continue;
            }
            let path = self
                .run_jj(&["workspace", "root", "--name", &name])
                .unwrap_or_default();
            let is_current = name == current_name;
            workspaces.push(WorkspaceInfo {
                name,
                path,
                is_current,
            });
        }

        Ok(workspaces)
    }

    /// Create a new workspace at the given path, optionally on a specific revision.
    pub fn workspace_add(&self, dest: &str, name: &str, rev: &str) -> CoreResult<String> {
        let mut args = vec!["workspace", "add", dest];
        if !name.is_empty() {
            args.extend(["--name", name]);
        }
        if !rev.is_empty() {
            args.extend(["-r", rev]);
        }
        let output = self.run_jj(&args)?;
        self.reload()?;
        Ok(output)
    }

    /// Remove a workspace.
    pub fn workspace_forget(&self, name: &str) -> CoreResult<()> {
        self.run_jj_reload(&["workspace", "forget", name])
    }
}
