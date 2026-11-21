use socket2::{Domain, SockAddr, Socket, Type};
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn session_ssh_auth_sock() -> Option<String> {
    let uid = unsafe { libc::geteuid() };
    let runtime = std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .unwrap_or_else(|| PathBuf::from(format!("/run/user/{uid}")));
    runtime_agent_socket(&runtime, uid, manager_agent_socket)?
        .into_os_string()
        .into_string()
        .ok()
}

fn manager_agent_socket(runtime: &Path) -> Option<PathBuf> {
    if !runtime.join("bus").exists() {
        return None;
    }
    let output = Command::new("/usr/bin/busctl")
        .env("XDG_RUNTIME_DIR", runtime)
        .args([
            "--user",
            "--timeout=1s",
            "--json=short",
            "call",
            "org.freedesktop.systemd1",
            "/org/freedesktop/systemd1",
            "org.freedesktop.DBus.Properties",
            "Get",
            "ss",
            "org.freedesktop.systemd1.Manager",
            "Environment",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let reply: serde_json::Value = serde_json::from_slice(&output.stdout).ok()?;
    reply["data"][0]["data"]
        .as_array()?
        .iter()
        .find_map(|entry| {
            let path = Path::new(entry.as_str()?.strip_prefix("SSH_AUTH_SOCK=")?);
            path.is_absolute().then(|| path.to_owned())
        })
}

fn runtime_agent_socket(
    runtime: &Path,
    uid: u32,
    manager_socket: impl FnOnce(&Path) -> Option<PathBuf>,
) -> Option<PathBuf> {
    let dir = fs::metadata(runtime).ok()?;
    if !dir.is_dir() || dir.uid() != uid || dir.mode() & 0o077 != 0 {
        return None;
    }
    let conventional = [
        "gcr/ssh",
        "keyring/ssh",
        "openssh_agent",
        "ssh-agent.socket",
    ]
    .iter()
    .map(|name| runtime.join(name));
    manager_socket(runtime)
        .into_iter()
        .chain(conventional)
        .find(|path| {
            fs::metadata(path).is_ok_and(|m| m.uid() == uid && m.file_type().is_socket())
                && listening_socket(path).is_ok()
        })
}

fn listening_socket(path: &Path) -> std::io::Result<()> {
    let socket = Socket::new(Domain::UNIX, Type::STREAM, None)?;
    socket.set_nonblocking(true)?;
    socket.connect(&SockAddr::unix(path)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::net::UnixListener;

    #[test]
    fn discovers_user_agent_in_private_runtime_directory() {
        let dir = tempfile::tempdir().unwrap();
        let runtime = dir.path();
        fs::set_permissions(runtime, fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir(runtime.join("gcr")).unwrap();
        drop(UnixListener::bind(runtime.join("gcr/ssh")).unwrap());
        let agent = runtime.join("openssh_agent");
        let _listener = UnixListener::bind(&agent).unwrap();
        let uid = fs::metadata(runtime).unwrap().uid();
        let selected = runtime.join("custom agent");
        let _selected = UnixListener::bind(&selected).unwrap();
        assert_eq!(
            runtime_agent_socket(runtime, uid, |_| Some(selected.clone())),
            Some(selected)
        );
        assert_eq!(runtime_agent_socket(runtime, uid, |_| None), Some(agent));

        fs::set_permissions(runtime, fs::Permissions::from_mode(0o777)).unwrap();
        assert_eq!(runtime_agent_socket(runtime, uid, |_| None), None);
    }
}
