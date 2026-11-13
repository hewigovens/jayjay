use super::connection::{self, Connection};
use super::helper::Helper;
use super::{RemoteDirectory, RemoteLocation};
use crate::{JayError, JayResult};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::sync::{
    Arc, LazyLock, Mutex,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostPlatform {
    MacOS,
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
}

static CONNECTIONS: LazyLock<Mutex<HashMap<String, Arc<MachineConnection>>>> =
    LazyLock::new(Mutex::default);
static SETUPS: LazyLock<Mutex<HashMap<String, Arc<Mutex<()>>>>> = LazyLock::new(Mutex::default);

pub(crate) struct MachineConnection {
    connection: Mutex<Arc<Connection>>,
    platform: HostPlatform,
    connecting: AtomicBool,
}
struct Connecting<'a>(&'a AtomicBool);
impl Drop for Connecting<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl MachineConnection {
    pub fn platform(&self) -> HostPlatform {
        self.platform
    }
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
        let host = Host::probe(machine)?;
        let helper = Helper::locate(host.target)?;
        let command = if host.platform == HostPlatform::Windows {
            helper.deploy_windows(machine)?
        } else {
            helper.deploy_unix(machine)?
        };
        let connection = Connection::connect(machine, &command)?;
        let shared = match &existing {
            Some(shared) => {
                *shared.connection.lock().unwrap() = connection;
                shared.clone()
            }
            None => Arc::new(MachineConnection {
                connection: Mutex::new(connection),
                platform: host.platform,
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

struct Host {
    target: &'static str,
    platform: HostPlatform,
}

impl Host {
    /// Unix hosts answer the `sh` probe; Windows hosts with a cmd or PowerShell login shell fail it and are asked through `cmd`.
    fn probe(machine: &str) -> JayResult<Self> {
        let unix = connection::run(
            machine,
            "sh -s",
            b"set -eu\nprintf '%s-%s' \"$(uname -s)\" \"$(uname -m)\"\n",
        );
        if let Some(host) = unix.as_deref().ok().and_then(Self::unix) {
            return Ok(host);
        }
        if let Err(error @ JayError::Refused { .. }) = unix {
            return Err(error);
        }
        let host = match connection::run(machine, "cmd /c echo %OS%-%PROCESSOR_ARCHITECTURE%", b"")
        {
            Ok(output) => Self::windows(&output),
            Err(_) => {
                return unix.and(Err(JayError::refused(
                    jayjay_i18n::ssh::error_unsupported_platform(),
                )));
            }
        };
        host.ok_or_else(|| JayError::refused(jayjay_i18n::ssh::error_unsupported_platform()))
    }

    fn unix(output: &str) -> Option<Self> {
        let (target, platform) = match output.trim() {
            "Darwin-arm64" => ("aarch64-apple-darwin", HostPlatform::MacOS),
            "Darwin-x86_64" => ("x86_64-apple-darwin", HostPlatform::MacOS),
            "Linux-aarch64" => ("aarch64-unknown-linux-musl", HostPlatform::Linux),
            "Linux-x86_64" => ("x86_64-unknown-linux-musl", HostPlatform::Linux),
            _ => return None,
        };
        Some(Self { target, platform })
    }

    fn windows(output: &str) -> Option<Self> {
        // A cmd login shell running `cmd /c` leaves a stray quote after the output.
        let target = match output.trim().trim_end_matches('"').trim_end() {
            "Windows_NT-ARM64" => "aarch64-pc-windows-msvc",
            "Windows_NT-AMD64" => "x86_64-pc-windows-msvc",
            _ => return None,
        };
        Some(Self {
            target,
            platform: HostPlatform::Windows,
        })
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

#[cfg(test)]
mod tests {
    use super::{Host, HostPlatform};

    #[test]
    fn probes_pick_the_helper_for_unix_and_windows_hosts() {
        assert_eq!(
            Host::unix("Linux-aarch64\n").map(|host| (host.target, host.platform)),
            Some(("aarch64-unknown-linux-musl", HostPlatform::Linux))
        );
        assert_eq!(
            Host::windows("Windows_NT-ARM64\"\r\n").map(|host| (host.target, host.platform)),
            Some(("aarch64-pc-windows-msvc", HostPlatform::Windows))
        );
        assert_eq!(
            Host::windows("Windows_NT-AMD64\r\n").map(|host| host.target),
            Some("x86_64-pc-windows-msvc")
        );
        assert!(Host::unix("MINGW64_NT-10.0-26100-x86_64").is_none());
        assert!(Host::windows("%OS%-%PROCESSOR_ARCHITECTURE%").is_none());
    }
}
