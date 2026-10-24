use std::path::Path;

pub fn file_manager_name() -> String {
    jayjay_i18n::settings::file_explorer()
}

pub fn custom_terminal_label() -> String {
    jayjay_i18n::settings::terminal_command_powershell()
}
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
