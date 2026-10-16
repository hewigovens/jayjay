use std::ffi::{OsStr, OsString};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{LocalFree, RPC_E_CHANGED_MODE};
use windows_sys::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
};
use windows_sys::Win32::UI::Shell::{
    CommandLineToArgvW, SHOpenFolderAndSelectItems, SHParseDisplayName, ShellExecuteW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

struct ShellApartment(bool);

impl ShellApartment {
    fn enter() -> Option<Self> {
        // SAFETY: Drop uninitializes only when this call initialized COM.
        let result = unsafe { CoInitializeEx(null(), COINIT_APARTMENTTHREADED as u32) };
        (result >= 0 || result == RPC_E_CHANGED_MODE).then_some(Self(result >= 0))
    }
}

impl Drop for ShellApartment {
    fn drop(&mut self) {
        if self.0 {
            // SAFETY: paired with a successful CoInitializeEx on this thread.
            unsafe {
                CoUninitialize();
            }
        }
    }
}

pub fn command_words(command: &str) -> Option<Vec<String>> {
    // CommandLineToArgvW parses argv[0] by different rules, so a placeholder takes that slot.
    let command = wide(format!("jayjay {command}"))?;
    // SAFETY: NUL-terminated input; `count` bounds the array, freed once below.
    unsafe {
        let mut count = 0;
        let argv = CommandLineToArgvW(command.as_ptr(), &mut count);
        if argv.is_null() {
            return None;
        }
        let words = std::slice::from_raw_parts(argv, count as usize)
            .iter()
            .skip(1)
            .map(|word| {
                let mut length = 0;
                while *(*word).add(length) != 0 {
                    length += 1;
                }
                String::from_utf16_lossy(std::slice::from_raw_parts(*word, length))
            })
            .collect();
        LocalFree(argv.cast());
        Some(words)
    }
}

fn wide(value: impl AsRef<OsStr>) -> Option<Vec<u16>> {
    let mut value: Vec<_> = value.as_ref().encode_wide().collect();
    if value.contains(&0) {
        return None;
    }
    value.push(0);
    Some(value)
}

/// `Some(None)` for an absent value; `None` only when a present value has an interior NUL.
fn optional_wide(value: Option<&str>) -> Option<Option<Vec<u16>>> {
    value.map_or(Some(None), |value| wide(value).map(Some))
}

fn ptr_or_null(value: &Option<Vec<u16>>) -> *const u16 {
    value.as_ref().map_or(null(), |value| value.as_ptr())
}

/// Shell calls can pump this thread's messages and re-enter GPUI mid-update.
pub(super) fn detached(task: impl FnOnce() -> bool + Send + 'static) -> bool {
    std::thread::Builder::new()
        .name("jayjay-shell".to_owned())
        .spawn(move || {
            task();
        })
        .is_ok()
}

pub fn open(target: impl AsRef<OsStr>) -> bool {
    let target = simplified(target.as_ref());
    detached(move || shell_execute(None, &target, None))
}

pub(super) fn launch(program: impl AsRef<OsStr> + Send + 'static, arguments: String) -> bool {
    detached(move || shell_execute(None, program.as_ref(), Some(&arguments)))
}

/// The `edit` verb never runs the file; false when the type has none.
pub(super) fn edit(path: &str) -> bool {
    shell_execute(Some("edit"), OsStr::new(path), None)
}

fn shell_execute(verb: Option<&str>, target: &OsStr, arguments: Option<&str>) -> bool {
    let Some(_apartment) = ShellApartment::enter() else {
        return false;
    };
    let (Some(target), Some(verb), Some(arguments)) =
        (wide(target), optional_wide(verb), optional_wide(arguments))
    else {
        return false;
    };
    // SAFETY: NUL-terminated buffers outlive the call.
    unsafe {
        ShellExecuteW(
            null_mut(),
            ptr_or_null(&verb),
            target.as_ptr(),
            ptr_or_null(&arguments),
            null(),
            SW_SHOWNORMAL,
        ) as isize
            > 32
    }
}

pub fn reveal(path: &Path) -> bool {
    let path = simplified(path.as_os_str());
    detached(move || {
        if Path::new(&path).is_dir() {
            shell_execute(None, &path, None)
        } else {
            select_in_folder(&path)
        }
    })
}

fn simplified(target: &OsStr) -> OsString {
    dunce::simplified(Path::new(target)).as_os_str().to_owned()
}

fn select_in_folder(path: &OsStr) -> bool {
    let Some(_apartment) = ShellApartment::enter() else {
        return false;
    };
    let Some(path) = wide(path) else {
        return false;
    };
    // SAFETY: COM is initialized and the PIDL is freed after use.
    unsafe {
        let mut item = null_mut();
        let parsed = SHParseDisplayName(path.as_ptr(), null_mut(), &mut item, 0, null_mut());
        let opened = parsed >= 0 && SHOpenFolderAndSelectItems(item, 0, null(), 0) >= 0;
        if !item.is_null() {
            CoTaskMemFree(item.cast());
        }
        opened
    }
}
