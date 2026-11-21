use std::process::Command;

pub fn launchctl_ssh_auth_sock() -> Option<String> {
    if let Some(socket) = launchctl(&["getenv", "SSH_AUTH_SOCK"])
        && !socket.trim().is_empty()
    {
        return Some(socket.trim().to_owned());
    }

    // An SSH login has its own launchd context; the user's agent lives in the GUI domain.
    let uid = unsafe { libc::getuid() };
    let domain = launchctl(&["print", &format!("gui/{uid}")])?;
    gui_agent_socket(&domain).map(str::to_owned)
}

fn launchctl(args: &[&str]) -> Option<String> {
    let output = Command::new("/bin/launchctl").args(args).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

fn gui_agent_socket(domain: &str) -> Option<&str> {
    let mut lines = domain.lines().map(str::trim);
    lines.find(|line| *line == "environment = {")?;
    lines
        .take_while(|line| *line != "}")
        .find_map(|line| line.strip_prefix("SSH_AUTH_SOCK => "))
        .filter(|path| !path.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_agent_socket_from_gui_domain_environment() {
        let domain = "gui/501 = {\n services = {\n SSH_AUTH_SOCK => /wrong\n }\n environment = {\n SSH_AUTH_SOCK => /agent with spaces\n }\n}";
        assert_eq!(gui_agent_socket(domain), Some("/agent with spaces"));
        assert_eq!(
            gui_agent_socket(&domain.replace("SSH_AUTH_SOCK => /agent with spaces", "")),
            None
        );
    }
}
