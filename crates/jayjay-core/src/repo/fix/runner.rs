use std::io::Write as _;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Mutex;

use futures::AsyncReadExt as _;
use jj_lib::backend::FileId;
use jj_lib::fix::{FileToFix, FixError};
use jj_lib::repo_path::RepoPath;
use jj_lib::store::Store;

use super::tools::FixTool;
use crate::repo::support::block_on;
use crate::types::FixToolFailure;

pub(super) struct FixRunner {
    tools: Vec<FixTool>,
    workspace_root: PathBuf,
    failures: Mutex<Vec<FixToolFailure>>,
}

impl FixRunner {
    pub(super) fn new(tools: Vec<FixTool>, workspace_root: PathBuf) -> Self {
        Self {
            tools,
            workspace_root,
            failures: Mutex::default(),
        }
    }

    pub(super) fn take_failures(&self) -> Vec<FixToolFailure> {
        let mut failures = std::mem::take(&mut *self.failures.lock().unwrap());
        failures.sort_by(|a, b| (&a.tool, &a.path).cmp(&(&b.tool, &b.path)));
        failures
    }

    pub(super) fn fix_file(
        &self,
        store: &Store,
        file: &FileToFix,
    ) -> Result<Option<FileId>, FixError> {
        let tools: Vec<&FixTool> = self
            .tools
            .iter()
            .filter(|tool| tool.matcher.matches(&file.repo_path))
            .collect();
        if tools.is_empty() {
            return Ok(None);
        }
        block_on(self.fix_content(store, file, &tools))
    }

    async fn fix_content(
        &self,
        store: &Store,
        file: &FileToFix,
        tools: &[&FixTool],
    ) -> Result<Option<FileId>, FixError> {
        let old_content = read_file(store, &file.repo_path, &file.file_id).await?;
        if old_content.is_empty() {
            return Ok(None);
        }
        let base_content = match &file.base_file_id {
            Some(base_id) if tools.iter().any(|tool| tool.line_range_arg.is_some()) => {
                Some(read_file(store, &file.repo_path, base_id).await?)
            }
            _ => None,
        };
        let mut content = old_content.clone();
        for tool in tools {
            match self.run_tool(tool, file, base_content.as_deref(), &content) {
                Ok(Some(output)) => content = output,
                Ok(None) => {}
                Err(message) => self.failures.lock().unwrap().push(FixToolFailure {
                    tool: tool.name.clone(),
                    path: file.repo_path.as_internal_file_string().to_owned(),
                    message,
                }),
            }
        }
        if content == old_content {
            return Ok(None);
        }
        Ok(Some(
            store
                .write_file(&file.repo_path, &mut content.as_slice())
                .await?,
        ))
    }

    fn run_tool(
        &self,
        tool: &FixTool,
        file: &FileToFix,
        base_content: Option<&[u8]>,
        content: &[u8],
    ) -> Result<Option<Vec<u8>>, String> {
        let mut variables = vec![("path", file.repo_path.as_internal_file_string())];
        if let Some(root) = self.workspace_root.to_str() {
            variables.push(("root", root));
        }
        let Some(mut command) = tool.command_for(&variables, base_content, content) else {
            return Ok(None);
        };
        let mut child = command
            .current_dir(&self.workspace_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("failed to start: {error}"))?;
        let mut stdin = child
            .stdin
            .take()
            .expect("child is spawned with piped stdin");
        let output = std::thread::scope(|scope| {
            scope.spawn(move || {
                // A tool that ignores stdin exits first; the broken pipe is not a failure.
                let _ = stdin.write_all(content);
            });
            child.wait_with_output()
        })
        .map_err(|error| format!("failed to wait: {error}"))?;
        if output.status.success() {
            return Ok(Some(output.stdout));
        }
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        Err(if stderr.is_empty() {
            output.status.to_string()
        } else {
            stderr
        })
    }
}

async fn read_file(store: &Store, path: &RepoPath, id: &FileId) -> Result<Vec<u8>, FixError> {
    let mut reader = store.read_file(path, id).await?;
    let mut content = Vec::new();
    reader.read_to_end(&mut content).await?;
    Ok(content)
}
