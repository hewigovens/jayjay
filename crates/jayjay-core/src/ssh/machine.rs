use super::connection::{self, Connection};
use super::{RemoteDirectory, RemoteLocation};
use crate::{JayError, JayResult};
use serde::de::DeserializeOwned;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{
    Arc, LazyLock, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

static HELPERS: OnceLock<PathBuf> = OnceLock::new();
static CONNECTIONS: LazyLock<Mutex<HashMap<String, Arc<MachineConnection>>>> =
    LazyLock::new(Mutex::default);
static SETUPS: LazyLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = LazyLock::new(Mutex::default);

pub fn set_helper_directory(path: PathBuf) {
    let _ = HELPERS.set(path);
}
pub fn helper_directory() -> PathBuf {
    HELPERS
        .get()
        .cloned()
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("../Resources/ssh-helpers")))
        })
        .unwrap_or_default()
}

pub(crate) struct MachineConnection {
    connection: Mutex<Arc<Connection>>,
    connecting: AtomicBool,
}
struct Connecting<'a>(&'a AtomicBool);
impl Drop for Connecting<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl MachineConnection {
    pub fn snapshot(&self) -> Arc<Connection> {
        self.connection.lock().unwrap().clone()
    }
    pub fn state(&self) -> ConnectionState {
        if self.connecting.load(Ordering::Acquire) {
            ConnectionState::Connecting
        } else if self.snapshot().connected() {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        }
    }
    pub fn connect(machine: &str) -> JayResult<Arc<Self>> {
        // Setup runs outside the global map lock so one slow host does not stall the others.
        let setup = SETUPS
            .lock()
            .unwrap()
            .entry(machine.into())
            .or_default()
            .clone();
        let _setup = setup.lock().unwrap();
        let existing = {
            let mut connections = CONNECTIONS.lock().unwrap();
            connections
                .retain(|_, shared| Arc::strong_count(shared) > 1 || shared.snapshot().connected());
            let existing = connections.get(machine).cloned();
            if let Some(shared) = &existing
                && shared.snapshot().connected()
            {
                return Ok(shared.clone());
            }
            if existing.is_none() && connections.len() >= 64 {
                return Err(JayError::refused(
                    jayjay_i18n::ssh::error_too_many_machines(),
                ));
            }
            existing
        };
        let _connecting = existing.as_ref().map(|shared| {
            shared.connecting.store(true, Ordering::Release);
            Connecting(&shared.connecting)
        });
        let target = connection::run(
            machine,
            "sh -s",
            b"set -eu\nprintf '%s-%s' \"$(uname -s)\" \"$(uname -m)\"\n",
        )?;
        let target = match target.trim() {
            "Darwin-arm64" => "aarch64-apple-darwin",
            "Darwin-x86_64" => "x86_64-apple-darwin",
            "Linux-aarch64" => "aarch64-unknown-linux-musl",
            "Linux-x86_64" => "x86_64-unknown-linux-musl",
            _ => {
                return Err(JayError::refused(
                    jayjay_i18n::ssh::error_unsupported_platform(),
                ));
            }
        };
        let bundled = helper_directory().join(target).join("jayjay-ssh");
        let executable = if bundled.is_file() {
            bundled
        } else {
            let sibling = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.join("jayjay-ssh")))
                .unwrap_or_default();
            let native = ((cfg!(target_os = "macos") && target.ends_with("apple-darwin"))
                || (cfg!(target_os = "linux") && target.ends_with("linux-musl")))
                && target.starts_with(std::env::consts::ARCH);
            if native && sibling.is_file() {
                sibling
            } else {
                return Err(JayError::internal(format!(
                    "The SSH helper for {target} is missing. Build it with `just ssh-helper {target}` or install a release containing it."
                )));
            }
        };
        let bytes = std::fs::read(executable).map_err(JayError::internal)?;
        let hash = hex::encode(Sha256::digest(&bytes));
        let base = format!("\"$HOME/.local/share/jayjay/helpers/{hash}\"");
        let verify = format!(
            "verify() {{ if command -v sha256sum >/dev/null; then sum=$(sha256sum \"$1\"); else sum=$(shasum -a 256 \"$1\"); fi; [ \"${{sum%% *}}\" = {hash} ]; }}; base={base}"
        );
        let installed = connection::run(machine, &format!("sh -c '{verify}; [ -x \"$base/jayjay-ssh\" ] && verify \"$base/jayjay-ssh\" && printf installed'"), b"").unwrap_or_default();
        if installed != "installed" {
            connection::run(
                machine,
                &format!(
                    "sh -c 'set -eu; umask 077; {verify}; mkdir -p \"$base\"; temporary=\"$base/.upload.$$\"; trap \"rm -f \\\"\\$temporary\\\"\" 0 1 2 15; cat > \"$temporary\"; verify \"$temporary\"; chmod 700 \"$temporary\"; mv \"$temporary\" \"$base/jayjay-ssh\"'"
                ),
                &bytes,
            )?;
        }
        let connection = Connection::connect(machine, &format!("{base}/jayjay-ssh"))?;
        let shared = match &existing {
            Some(shared) => {
                *shared.connection.lock().unwrap() = connection;
                shared.clone()
            }
            None => Arc::new(MachineConnection {
                connection: Mutex::new(connection),
                connecting: AtomicBool::new(false),
            }),
        };
        CONNECTIONS
            .lock()
            .unwrap()
            .insert(machine.into(), shared.clone());
        Ok(shared)
    }
}

pub fn browse(machine: &str, path: &str) -> JayResult<RemoteDirectory> {
    request(&RemoteLocation::new(machine, path)?, "__browse").map(|(_, directory)| directory)
}

pub(crate) fn init_repository(location: &RemoteLocation) -> JayResult<()> {
    request::<()>(location, "__init").map(drop)
}

pub(super) fn request<T: DeserializeOwned>(
    location: &RemoteLocation,
    method: &str,
) -> JayResult<(Arc<MachineConnection>, T)> {
    let shared = MachineConnection::connect(&location.machine)?;
    super::remember_machine(&location.machine);
    let value = shared
        .snapshot()
        .request(&location.path, method, vec![], None)?
        .result?;
    let value = serde_json::from_value(value).map_err(JayError::internal)?;
    Ok((shared, value))
}
