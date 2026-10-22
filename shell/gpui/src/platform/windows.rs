use std::path::Path;

pub const SHOW_IN_FILE_MANAGER_LABEL: &str = "Show in File Explorer";
pub const CUSTOM_TERMINAL_LABEL: &str = "Command (accepts PowerShell arguments)";
pub const JJ_PACKAGE_MANAGER: (&str, &str) = ("WinGet", "winget install jj-vcs.jj");

pub fn open_url(target: &str) -> bool {
    jayjay_core::tools::windows_shell::open(target)
}

pub fn reveal_path(path: &Path) -> bool {
    jayjay_core::tools::windows_shell::reveal(path)
}

pub fn send_notification(_title: &str, _body: &str) -> bool {
    false
}

pub(super) fn numeric_locale() -> String {
    let mut name = [0_u16; 85];
    let length = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(
            name.as_mut_ptr(),
            name.len() as i32,
        )
    };
    if length <= 1 {
        return String::new();
    }
    String::from_utf16_lossy(&name[..length as usize - 1])
}
