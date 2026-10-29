use std::path::Path;

use gpui::Window;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, SC_KEYMENU, WM_SYSCOMMAND};

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

/// Posted rather than sent so the modal menu loop starts after GPUI finishes dispatching the click.
pub(crate) fn show_system_menu(window: &Window) {
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    unsafe {
        PostMessageW(
            handle.hwnd.get() as _,
            WM_SYSCOMMAND,
            SC_KEYMENU as usize,
            b' ' as isize,
        )
    };
}
