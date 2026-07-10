use std::env;
use std::path::{Path, PathBuf};

// Search order matches app::find_app_executable: sibling executable first, then PATH.
pub(super) fn find_gpui_executable() -> Option<PathBuf> {
    sibling_gpui_executable().or_else(|| find_in_path(&gpui_executable_name()))
}

fn sibling_gpui_executable() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let exe = exe.canonicalize().unwrap_or(exe);
    sibling_gpui_executable_for_cli(&exe)
}

fn sibling_gpui_executable_for_cli(cli: &Path) -> Option<PathBuf> {
    let candidate = cli.parent()?.join(gpui_executable_name());
    candidate.is_file().then_some(candidate)
}

fn find_in_path(name: &str) -> Option<PathBuf> {
    let path_var = env::var_os("PATH")?;
    find_in_dirs(name, env::split_paths(&path_var))
}

fn find_in_dirs(name: &str, dirs: impl Iterator<Item = PathBuf>) -> Option<PathBuf> {
    dirs.map(|dir| dir.join(name))
        .find(|candidate| candidate.is_file())
}

fn gpui_executable_name() -> String {
    executable_name_with_suffix(env::consts::EXE_SUFFIX)
}

// Takes the suffix directly, rather than reading env::consts, so the Windows `.exe` naming is unit-testable without a Windows target.
fn executable_name_with_suffix(suffix: &str) -> String {
    format!("jayjay-gpui{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_sibling_gpui_executable_is_next_to_cli() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let cli = temp_dir.path().join("jayjay-cli");
        let gpui = temp_dir.path().join(gpui_executable_name());
        fs::write(&cli, "").expect("write cli");
        fs::write(&gpui, "").expect("write gpui");

        assert_eq!(sibling_gpui_executable_for_cli(&cli), Some(gpui));
    }

    #[test]
    fn test_sibling_gpui_executable_absent() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let cli = temp_dir.path().join("jayjay-cli");
        fs::write(&cli, "").expect("write cli");

        assert_eq!(sibling_gpui_executable_for_cli(&cli), None);
    }

    #[test]
    fn test_find_in_dirs_returns_first_match() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir).expect("create empty dir");
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir(&bin_dir).expect("create bin dir");
        let target = bin_dir.join("jayjay-gpui");
        fs::write(&target, "").expect("write target");

        let found = find_in_dirs("jayjay-gpui", [empty_dir, bin_dir].into_iter());
        assert_eq!(found, Some(target));
    }

    #[test]
    fn test_find_in_dirs_none_when_absent() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let found = find_in_dirs("jayjay-gpui", [temp_dir.path().to_path_buf()].into_iter());
        assert_eq!(found, None);
    }

    #[test]
    fn test_find_in_dirs_matches_exe_suffixed_name() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let target = temp_dir.path().join("jayjay-gpui.exe");
        fs::write(&target, "").expect("write target");

        let found = find_in_dirs(
            "jayjay-gpui.exe",
            [temp_dir.path().to_path_buf()].into_iter(),
        );
        assert_eq!(found, Some(target));
    }

    #[test]
    fn test_executable_name_has_no_suffix_on_unix() {
        assert_eq!(executable_name_with_suffix(""), "jayjay-gpui");
    }

    #[test]
    fn test_executable_name_appends_exe_suffix_on_windows() {
        assert_eq!(executable_name_with_suffix(".exe"), "jayjay-gpui.exe");
    }
}
