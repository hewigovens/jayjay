use gix::bstr::BStr;
use gix::validate::reference::name_partial;

use crate::types::{JayError, JayResult};

/// Git's own short-ref rules, plus no leading `-` so a name never reads as a flag.
fn ref_name_problem(name: &str) -> Option<String> {
    if name.starts_with('-') {
        return Some("Reference name cannot start with '-'".to_owned());
    }
    name_partial(BStr::new(name))
        .err()
        .map(|error| error.to_string())
}

pub fn is_valid_bookmark_name(name: &str) -> bool {
    ref_name_problem(name).is_none()
}

pub(super) fn ensure_valid_ref_name(kind: &str, name: &str) -> JayResult<()> {
    match ref_name_problem(name) {
        None => Ok(()),
        Some(problem) => Err(JayError::internal(format!(
            "\"{name}\" is not a valid {kind} name: {problem}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_bookmark_names() {
        for ok in [
            "feat-add-x-abc123",
            "user/feat/thing",
            "v1.2-rc",
            &"b".repeat(255),
        ] {
            assert!(is_valid_bookmark_name(ok), "{ok} should be valid");
        }
        for bad in [
            "",
            "@",
            "has space",
            "bad..dots",
            "-leading-dash",
            "trailing/",
            "ends.",
            "name.lock",
            "foo.lock/bar",
            "ti~lde",
            "co:lon",
            "a//b",
            "foo/.hidden",
            "ctrl\tchar",
            "/leading-slash",
            "a@{upstream}",
        ] {
            assert!(!is_valid_bookmark_name(bad), "{bad:?} should be invalid");
        }
    }
}
