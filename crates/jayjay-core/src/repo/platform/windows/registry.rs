use windows_sys::Win32::Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RegGetValueW,
};

/// The PATH a new sign-in would get. A launcher can hand JayJay a narrower one than the user's, as PowerToys Run does.
pub fn registry_path() -> Option<String> {
    let path = [
        (
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
        (HKEY_CURRENT_USER, "Environment"),
    ]
    .into_iter()
    .filter_map(|(root, key)| registry_string(root, key, "Path"))
    .collect::<Vec<_>>()
    .join(";");
    (!path.is_empty()).then_some(path)
}

fn registry_string(root: HKEY, key: &str, value: &str) -> Option<String> {
    let wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
    let (key, value) = (wide(key), wide(value));
    let mut buffer = vec![0u16; 1024];
    loop {
        let mut bytes = (buffer.len() * 2) as u32;
        // RRF_RT_REG_SZ alone also returns a REG_EXPAND_SZ value, expanded; naming both types is refused before Windows 8.
        // SAFETY: NUL-terminated names; `bytes` bounds the buffer going in and reports the size used or needed.
        let status = unsafe {
            RegGetValueW(
                root,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buffer.as_mut_ptr().cast(),
                &mut bytes,
            )
        };
        match status {
            ERROR_SUCCESS => {
                let text = &buffer[..(bytes as usize / 2).min(buffer.len())];
                let end = text
                    .iter()
                    .position(|&unit| unit == 0)
                    .unwrap_or(text.len());
                return Some(String::from_utf16_lossy(&text[..end]))
                    .filter(|text| !text.is_empty());
            }
            // The size reported for a value with variables to expand can fall short, so the buffer at least doubles.
            ERROR_MORE_DATA => buffer.resize((bytes as usize / 2 + 1).max(buffer.len() * 2), 0),
            _ => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::system_root;
    use super::*;

    #[test]
    fn registry_string_expands_a_value_longer_than_its_first_buffer() {
        use windows_sys::Win32::System::Registry::{REG_EXPAND_SZ, RegDeleteKeyW, RegSetKeyValueW};

        let wide = |text: &str| text.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let key = format!(r"Software\JayJayTest-{}", std::process::id());
        let tail = "x".repeat(1500);
        let (wide_key, name, data) = (
            wide(&key),
            wide("Path"),
            wide(&format!(r"%SystemRoot%\{tail}")),
        );
        // SAFETY: NUL-terminated strings; the byte count covers `data` with its terminator.
        let written = unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                wide_key.as_ptr(),
                name.as_ptr(),
                REG_EXPAND_SZ,
                data.as_ptr().cast(),
                (data.len() * 2) as u32,
            )
        };
        assert_eq!(written, ERROR_SUCCESS);

        let read = registry_string(HKEY_CURRENT_USER, &key, "Path");
        // SAFETY: NUL-terminated name of the key this test created.
        unsafe { RegDeleteKeyW(HKEY_CURRENT_USER, wide_key.as_ptr()) };

        let expected = format!(r"{}\{tail}", system_root().display());
        assert_eq!(read.as_deref(), Some(expected.as_str()));
        assert_eq!(registry_string(HKEY_CURRENT_USER, &key, "Path"), None);
    }
}
