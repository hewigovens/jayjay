use std::fs;
use std::path::{Path, PathBuf};

use jayjay_primitives::hex_sha256;
use parking_lot::Mutex;

use crate::filesystem::io_error;
use crate::{JayError, JayResult, write_atomically};

const SKILL: &str = include_str!("../../../../skills/jayjay/SKILL.md");
const MARKER: &str = "\n<!-- Managed by JayJay: ";
static INSTALL_LOCK: Mutex<()> = Mutex::new(());

pub struct AgentSkillInstallation {
    pub installed: bool,
    pub complete: bool,
}

pub struct AgentSkillInstaller {
    home: PathBuf,
}

impl AgentSkillInstaller {
    pub fn new(home: PathBuf) -> Self {
        Self { home }
    }

    pub fn for_user() -> JayResult<Self> {
        let home = std::env::var_os("JAYJAY_SKILLS_HOME")
            .filter(|home| !home.is_empty())
            .map(PathBuf::from)
            .or_else(crate::home_dir)
            .ok_or_else(|| JayError::internal("cannot locate the home directory"))?;
        Ok(Self::new(home))
    }

    pub fn status(&self) -> JayResult<AgentSkillInstallation> {
        let _lock = INSTALL_LOCK.lock();
        let states = self.states()?;
        Ok(AgentSkillInstallation {
            installed: states
                .iter()
                .any(|(_, state)| matches!(state, SkillState::Managed(_))),
            complete: states.iter().all(|(_, state)| state.is_current()),
        })
    }

    pub fn set_installed(&self, installed: bool) -> JayResult<()> {
        let _lock = INSTALL_LOCK.lock();
        let states = self.states()?;
        if installed {
            if let Some((path, _)) = states
                .iter()
                .find(|(_, state)| matches!(state, SkillState::Unmanaged))
            {
                return Err(JayError::refused(
                    jayjay_i18n::settings::agent_skill_conflict(&path.display().to_string()),
                ));
            }
            let content = managed_file(SKILL);
            for (path, state) in states {
                if state.is_current() {
                    continue;
                }
                write_atomically(&path.join("SKILL.md"), content.as_bytes())
                    .map_err(|error| io_error("install skill at", &path, error))?;
            }
        } else {
            for (path, _) in states {
                if matches!(SkillState::read(&path)?, SkillState::Managed(_)) {
                    fs::remove_file(path.join("SKILL.md"))
                        .map_err(|error| io_error("remove skill at", &path, error))?;
                    if let Err(error) = fs::remove_dir(&path)
                        && error.kind() != std::io::ErrorKind::DirectoryNotEmpty
                    {
                        return Err(io_error("remove skill directory", &path, error));
                    }
                }
            }
        }
        Ok(())
    }

    fn states(&self) -> JayResult<Vec<(PathBuf, SkillState)>> {
        // Codex and Cursor share .agents; Claude Code also needs its own discovery directory.
        [".agents/skills/jayjay", ".claude/skills/jayjay"]
            .into_iter()
            .map(|relative| {
                let path = self.home.join(relative);
                let state = SkillState::read(&path)?;
                Ok((path, state))
            })
            .collect()
    }
}

pub(super) fn managed_file(content: &str) -> String {
    format!("{content}{MARKER}{} -->\n", hex_sha256(content.as_bytes()))
}

enum SkillState {
    Missing,
    Managed(String),
    Unmanaged,
}

impl SkillState {
    fn is_current(&self) -> bool {
        matches!(self, Self::Managed(content) if content == SKILL)
    }

    fn read(directory: &Path) -> JayResult<Self> {
        match fs::symlink_metadata(directory) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Self::Missing),
            Err(error) => return Err(io_error("inspect skill directory", directory, error)),
            Ok(metadata) if !metadata.file_type().is_dir() => return Ok(Self::Unmanaged),
            Ok(_) => {}
        }
        let path = directory.join("SKILL.md");
        match fs::symlink_metadata(&path) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Self::Missing),
            Err(error) => return Err(io_error("inspect skill", &path, error)),
            Ok(metadata) if !metadata.file_type().is_file() => return Ok(Self::Unmanaged),
            Ok(_) => {}
        }
        let bytes = fs::read(&path).map_err(|error| io_error("read skill", &path, error))?;
        let Ok(text) = String::from_utf8(bytes) else {
            return Ok(Self::Unmanaged);
        };
        // A checksum keeps app updates recognizable while preserving copies the user edited.
        if let Some((content, checksum)) = text.rsplit_once(MARKER)
            && checksum.strip_suffix(" -->\n") == Some(hex_sha256(content.as_bytes()).as_str())
        {
            return Ok(Self::Managed(content.to_owned()));
        }
        Ok(Self::Unmanaged)
    }
}
