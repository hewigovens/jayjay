#[cfg(target_os = "macos")]
mod macos;

#[cfg(all(unix, not(target_os = "macos")))]
mod linux;
#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(all(unix, not(target_os = "macos")))]
pub use linux::launchctl_ssh_auth_sock;
#[cfg(target_os = "macos")]
pub use macos::launchctl_ssh_auth_sock;
#[cfg(unix)]
pub use unix::{command, executable_names, home_dir, is_executable_file, system_path_entries};
#[cfg(windows)]
pub use windows::{
    command, executable_names, home_dir, is_executable_file, launchctl_ssh_auth_sock,
    system_path_entries, system32,
};
