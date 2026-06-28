use std::process::Command;

pub fn launchctl_ssh_auth_sock() -> Option<String> {
    let output = Command::new("/bin/launchctl")
        .args(["getenv", "SSH_AUTH_SOCK"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!value.is_empty()).then_some(value)
}
