//! Host paths cross the SSH protocol `/`-rooted, which is how clients recognize them; Windows hosts write `C:\dir` as `/C:/dir`.

use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) fn to_native(wire: &str) -> PathBuf {
    #[cfg(windows)]
    if let Some(native) = windows_native(wire) {
        return native.into();
    }
    PathBuf::from(wire)
}

pub(crate) fn to_wire(path: &Path) -> String {
    let path = dunce::simplified(path).to_string_lossy();
    #[cfg(windows)]
    if let Some(wire) = windows_wire(&path) {
        return wire;
    }
    path.into_owned()
}

/// Rewrites the host paths in an `@paths` result for the wire.
pub(crate) fn wire_paths(value: &mut Value) {
    if let Value::String(path) = value {
        convert_absolute(path);
    } else {
        for_each_path(value, &mut convert_absolute);
    }
}

fn convert_absolute(path: &mut String) {
    if Path::new(path.as_str()).is_absolute() {
        *path = to_wire(Path::new(path.as_str()));
    }
}

/// Visits the fields of an `@paths` result that hold host paths.
pub(super) fn for_each_path(value: &mut Value, visit: &mut impl FnMut(&mut String)) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                match value {
                    Value::String(path)
                        if matches!(key.as_str(), "path" | "pinnable_path" | "dest") =>
                    {
                        visit(path)
                    }
                    value => for_each_path(value, visit),
                }
            }
        }
        Value::Array(values) => values
            .iter_mut()
            .for_each(|value| for_each_path(value, visit)),
        _ => {}
    }
}

fn drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

#[cfg_attr(not(windows), allow(dead_code))]
fn windows_native(wire: &str) -> Option<String> {
    wire.strip_prefix('/')
        .filter(|rest| drive_path(rest))
        .map(|rest| rest.replace('/', "\\"))
}

/// The wire form of a Windows host path, which Windows OpenSSH's file transfer also accepts.
pub(super) fn wire_form(native: &str) -> String {
    windows_wire(native).unwrap_or_else(|| native.into())
}

fn windows_wire(native: &str) -> Option<String> {
    drive_path(native).then(|| format!("/{}", native.replace('\\', "/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_paths_round_trip_through_the_wire_form() {
        for (native, wire) in [
            (r"C:\Users\worker\repo", "/C:/Users/worker/repo"),
            (r"D:\", "/D:/"),
            (r"C:\work\with space\#1", "/C:/work/with space/#1"),
        ] {
            assert_eq!(windows_wire(native).as_deref(), Some(wire));
            assert_eq!(windows_native(wire).as_deref(), Some(native));
        }
        for unix in ["/home/worker/repo", "/", "/c/repo"] {
            assert_eq!(windows_native(unix), None);
        }
        assert_eq!(windows_wire(r"\\server\share"), None);
    }
}
