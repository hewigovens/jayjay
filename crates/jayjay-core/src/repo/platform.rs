#[cfg(target_os = "macos")]
mod macos;

#[cfg(not(target_os = "macos"))]
mod linux;

#[cfg(not(target_os = "macos"))]
pub use linux::launchctl_ssh_auth_sock;
#[cfg(target_os = "macos")]
pub use macos::launchctl_ssh_auth_sock;
