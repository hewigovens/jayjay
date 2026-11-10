use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::{env, fs, thread};

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::repo::find_existing_binary;
use crate::tools::launcher::detach_stdio;

use super::app;

const TAB_POLLS: u32 = 50;
const TAB_POLL_INTERVAL: Duration = Duration::from_millis(200);

pub(super) struct Cli(String);

impl Cli {
    pub(super) fn find() -> Option<Self> {
        // The app's own copy first: another tool may also be called `rex`.
        app::bundled_cli()
            .or_else(|| find_existing_binary("rex"))
            .map(Self)
    }

    pub(super) fn run(&self, cwd: &str, command: &str) -> bool {
        match self.sessions().as_deref() {
            None => false,
            Some([session]) => {
                let ran = self.run_in(session, cwd, command);
                if ran {
                    app::open_app(&[]);
                }
                ran
            }
            Some(sessions) => self.run_in_shown_session(sessions, cwd, command) == Some(true),
        }
    }

    /// The CLI cannot see which of several sessions the app shows, but the app opens a folder as a tab in that one. A folder made for this launch is what proves the tab is ours to close.
    fn run_in_shown_session(&self, sessions: &[String], cwd: &str, command: &str) -> Option<bool> {
        let known: HashSet<String> = self
            .tabs(sessions)?
            .into_iter()
            .map(|tab| tab.block_id)
            .collect();
        let folder = LaunchFolder::create()?;
        app::open(folder.0.to_str()?).then_some(())?;
        let tab = self.tab_in(&folder.0, &known)?;
        let ran = self.run_in(&tab.session_id, cwd, command);
        let _ = detach_stdio(self.command().args([
            "block",
            "close",
            "--session",
            &tab.session_id,
            &tab.block_id,
        ]))
        .status();
        Some(ran)
    }

    fn run_in(&self, session_id: &str, cwd: &str, command: &str) -> bool {
        detach_stdio(self.command().args([
            "run",
            "--session",
            session_id,
            "--cwd",
            cwd,
            "--",
            "bash",
            "-c",
            command,
        ]))
        .status()
        .is_ok_and(|status| status.success())
    }

    fn tab_in(&self, folder: &Path, known: &HashSet<String>) -> Option<Tab> {
        for _ in 0..TAB_POLLS {
            thread::sleep(TAB_POLL_INTERVAL);
            let Some(tabs) = self.sessions().and_then(|sessions| self.tabs(&sessions)) else {
                continue;
            };
            let opened = tabs
                .into_iter()
                .filter(|tab| !known.contains(&tab.block_id))
                .find(|tab| self.working_directory(tab).as_deref() == Some(folder));
            if opened.is_some() {
                return opened;
            }
        }
        None
    }

    fn working_directory(&self, tab: &Tab) -> Option<PathBuf> {
        let process: Process = self.json(&[
            "block",
            "call",
            "--session",
            &tab.session_id,
            "--block",
            &tab.block_id,
            "process",
        ])?;
        process.foreground?.cwd.map(PathBuf::from)
    }

    fn sessions(&self) -> Option<Vec<String>> {
        let listed: Sessions = self.json(&["ls", "--json"])?;
        Some(
            listed
                .sessions
                .into_iter()
                .map(|session| session.session_id)
                .collect(),
        )
    }

    /// None when any session's tabs cannot be listed, so a known tab is never taken for a new one.
    fn tabs(&self, sessions: &[String]) -> Option<Vec<Tab>> {
        let mut tabs = Vec::new();
        for session_id in sessions {
            let listed: Blocks = self.json(&["block", "ls", "--session", session_id, "--json"])?;
            tabs.extend(listed.blocks.into_iter().map(|block| Tab {
                session_id: session_id.clone(),
                block_id: block.block_id,
            }));
        }
        Some(tabs)
    }

    fn json<T: DeserializeOwned>(&self, arguments: &[&str]) -> Option<T> {
        let output = self.command().args(arguments).output().ok()?;
        output.status.success().then_some(())?;
        serde_json::from_slice(&output.stdout).ok()
    }

    fn command(&self) -> Command {
        Command::new(&self.0)
    }
}

struct Tab {
    session_id: String,
    block_id: String,
}

struct LaunchFolder(PathBuf);

impl LaunchFolder {
    fn create() -> Option<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_nanos();
        // Rex reports a tab's working directory with symlinks resolved.
        let path = fs::canonicalize(env::temp_dir())
            .ok()?
            .join(format!("jayjay-rex-{}-{nanos}", process::id()));
        fs::create_dir(&path).ok()?;
        Some(Self(path))
    }
}

impl Drop for LaunchFolder {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.0);
    }
}

#[derive(Deserialize)]
struct Sessions {
    sessions: Vec<Session>,
}

#[derive(Deserialize)]
struct Session {
    session_id: String,
}

#[derive(Deserialize)]
struct Blocks {
    blocks: Vec<Block>,
}

#[derive(Deserialize)]
struct Block {
    block_id: String,
}

#[derive(Deserialize)]
struct Process {
    foreground: Option<Foreground>,
}

#[derive(Deserialize)]
struct Foreground {
    cwd: Option<String>,
}
