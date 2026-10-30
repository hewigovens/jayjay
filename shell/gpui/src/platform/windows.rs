use std::path::Path;

use gpui::{App, Pixels, Point, Window};
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::{HWND, POINT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnableMenuItem, GetSystemMenu, IsZoomed, MF_BYCOMMAND, MF_ENABLED, MF_GRAYED, PostMessageW,
    SC_CLOSE, SC_MAXIMIZE, SC_MOVE, SC_RESTORE, SC_SIZE, SetMenuDefaultItem, TPM_LEFTBUTTON,
    TPM_RETURNCMD, TrackPopupMenu, WM_SYSCOMMAND,
};

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

/// Shows the system menu with its top-left corner at `at`; the menu runs a modal loop, so it starts from a task rather than inside the click handler.
pub(crate) fn show_system_menu(at: Point<Pixels>, window: &Window, cx: &mut App) {
    let Ok(handle) = HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::Win32(handle) = handle.as_raw() else {
        return;
    };
    let hwnd = handle.hwnd.get() as HWND as isize;
    let scale = window.scale_factor();
    let mut point = POINT {
        x: (f32::from(at.x) * scale).round() as i32,
        y: (f32::from(at.y) * scale).round() as i32,
    };
    cx.spawn(async move |_| unsafe {
        let hwnd = hwnd as HWND;
        let menu = GetSystemMenu(hwnd, 0);
        if menu.is_null() || ClientToScreen(hwnd, &mut point) == 0 {
            return;
        }
        let maximized = IsZoomed(hwnd) != 0;
        for (command, enabled) in [
            (SC_RESTORE, maximized),
            (SC_MOVE, !maximized),
            (SC_SIZE, !maximized),
            (SC_MAXIMIZE, !maximized),
        ] {
            let state = if enabled { MF_ENABLED } else { MF_GRAYED };
            EnableMenuItem(menu, command, MF_BYCOMMAND | state);
        }
        SetMenuDefaultItem(menu, SC_CLOSE, 0);
        let command = TrackPopupMenu(
            menu,
            TPM_RETURNCMD | TPM_LEFTBUTTON,
            point.x,
            point.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        if command != 0 {
            PostMessageW(hwnd, WM_SYSCOMMAND, command as usize, 0);
        }
    })
    .detach();
}
