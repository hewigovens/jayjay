use super::*;
use std::fs;

fn write_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\n").expect("write file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}

// Sibling lookup composes names with EXE_SUFFIX, so fixtures must too or Windows looks for jayjay-cli.exe and misses.
fn suffixed(name: &str) -> String {
    format!("{name}{}", std::env::consts::EXE_SUFFIX)
}

#[test]
fn appimage_target_wins_over_sibling_cli() {
    let temp = tempfile::tempdir().expect("temp dir");
    let image = temp.path().join("JayJay.AppImage");
    write_executable(&image);
    let exe = temp.path().join("jayjay-gpui");
    write_executable(&exe);
    write_executable(&temp.path().join("jayjay-cli"));

    let target = resolve_target_from(Some(image.clone()), Some(exe));
    assert_eq!(target, Ok(image));
}

#[test]
fn missing_appimage_errors_instead_of_linking_mounted_exe() {
    let temp = tempfile::tempdir().expect("temp dir");
    let exe = temp.path().join("jayjay-gpui");
    write_executable(&exe);
    let gone = temp.path().join("Deleted.AppImage");

    let target = resolve_target_from(Some(gone), Some(exe));
    assert!(target.is_err_and(|err| err.contains("AppImage")));
}

#[test]
fn relative_appimage_path_is_rejected() {
    let target = resolve_target_from(Some(PathBuf::from("JayJay.AppImage")), None);
    assert!(target.is_err());
}

// The resolver canonicalizes current_exe, so these tests compare against canonicalized tempdir paths (macOS mounts tempdirs under the /var -> /private/var symlink).
#[test]
fn sibling_cli_is_preferred_over_gpui_binary() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path().canonicalize().expect("canonicalize");
    let exe = root.join("jayjay-gpui");
    write_executable(&exe);
    let cli = root.join(suffixed("jayjay-cli"));
    write_executable(&cli);

    assert_eq!(resolve_target_from(None, Some(exe)), Ok(cli));
}

#[test]
fn plain_binary_without_sibling_targets_itself() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path().canonicalize().expect("canonicalize");
    let exe = root.join("jayjay-gpui");
    write_executable(&exe);

    assert_eq!(resolve_target_from(None, Some(exe.clone())), Ok(exe));
}

#[cfg(unix)]
#[test]
fn own_installed_symlink_is_never_picked_as_sibling_target() {
    let temp = tempfile::tempdir().expect("temp dir");
    let root = temp.path().canonicalize().expect("canonicalize");
    let exe = root.join("jayjay-gpui");
    write_executable(&exe);
    // Simulates running from ~/.local/bin where our own `jayjay` link sits beside the binary.
    std::os::unix::fs::symlink(&exe, root.join(CLI_NAME)).expect("symlink");

    assert_eq!(resolve_target_from(None, Some(exe.clone())), Ok(exe));
}

#[test]
fn inspect_reports_missing_entry() {
    let temp = tempfile::tempdir().expect("temp dir");
    let status = inspect_entry(&temp.path().join(CLI_NAME));
    assert_eq!(status, EntryStatus::NotInstalled);
}

#[test]
fn inspect_reports_regular_file_as_unmanaged() {
    let temp = tempfile::tempdir().expect("temp dir");
    let entry = temp.path().join(CLI_NAME);
    write_executable(&entry);

    assert_eq!(
        inspect_entry(&entry),
        EntryStatus::Unmanaged { target: None }
    );
}

#[test]
fn uninstall_refuses_a_regular_file_entry() {
    let temp = tempfile::tempdir().expect("temp dir");
    let entry = temp.path().join(CLI_NAME);
    write_executable(&entry);

    let result = perform_uninstall(temp.path());
    assert!(result.is_err_and(|err| err.contains("remove it manually")));
    assert!(entry.exists(), "the user's own binary must survive");
}

#[cfg(unix)]
#[test]
fn foreign_symlink_is_unmanaged_and_uninstall_refuses() {
    let temp = tempfile::tempdir().expect("temp dir");
    let other = temp.path().join("other-tool");
    write_executable(&other);
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&other, &link).expect("symlink");

    assert_eq!(
        inspect_entry(&link),
        EntryStatus::Unmanaged {
            target: Some(other.clone())
        }
    );
    assert!(perform_uninstall(temp.path()).is_err());
    assert_eq!(fs::read_link(&link).expect("link must survive"), other);
}

#[cfg(unix)]
#[test]
fn live_symlink_matching_only_by_name_is_unmanaged() {
    let temp = tempfile::tempdir().expect("temp dir");
    for name in ["jayjay-cli", CLI_NAME] {
        let foreign = temp.path().join("custom").join(suffixed(name));
        fs::create_dir_all(foreign.parent().expect("parent")).expect("mkdir");
        write_executable(&foreign);
        let link = temp.path().join(format!("link-{name}"));
        std::os::unix::fs::symlink(&foreign, &link).expect("symlink");

        assert_eq!(
            inspect_entry_against(&link, &[temp.path().join(suffixed("jayjay-cli"))]),
            EntryStatus::Unmanaged {
                target: Some(foreign)
            },
            "{name}: live basename match must not classify as ours"
        );
    }
}

#[cfg(unix)]
#[test]
fn live_symlink_with_canonical_identity_is_installed() {
    let temp = tempfile::tempdir().expect("temp dir");
    let target = temp.path().join(suffixed("jayjay-cli"));
    write_executable(&target);
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&target, &link).expect("symlink");

    assert_eq!(
        inspect_entry_against(&link, std::slice::from_ref(&target)),
        EntryStatus::Installed { target }
    );
}

#[cfg(unix)]
#[test]
fn dangling_bare_jayjay_target_is_unmanaged() {
    let temp = tempfile::tempdir().expect("temp dir");
    let gone = temp.path().join("custom").join(CLI_NAME);
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&gone, &link).expect("symlink");

    assert_eq!(
        inspect_entry_against(&link, &[temp.path().join(suffixed("jayjay-cli"))]),
        EntryStatus::Unmanaged { target: Some(gone) }
    );
}

// A dangling link can't be matched by canonical path, so file-name matching must keep a link to a moved install recognized as ours.
#[cfg(unix)]
#[test]
fn broken_link_to_relocated_install_stays_managed() {
    let temp = tempfile::tempdir().expect("temp dir");
    let old = temp.path().join("old").join("jayjay-gpui");
    let new = temp.path().join("new").join("jayjay-gpui");
    fs::create_dir_all(new.parent().expect("parent")).expect("mkdir");
    write_executable(&new);
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&old, &link).expect("symlink");

    assert_eq!(
        inspect_entry_against(&link, &[new]),
        EntryStatus::Broken { target: old }
    );
}

// Repair and Reinstall only act on Broken entries, so classifying a foreign dangling link as Unmanaged proves it is left alone.
#[cfg(unix)]
#[test]
fn broken_foreign_symlink_is_unmanaged_not_repairable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let gone = temp.path().join("gone-tool");
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&gone, &link).expect("symlink");

    assert_eq!(
        inspect_entry(&link),
        EntryStatus::Unmanaged { target: Some(gone) }
    );
    assert!(perform_uninstall(temp.path()).is_err());
    assert!(link.symlink_metadata().is_ok(), "foreign link must survive");
}

#[cfg(unix)]
#[test]
fn broken_managed_symlink_is_removable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(temp.path().join("JayJay-Old.AppImage"), &link).expect("symlink");

    perform_uninstall(temp.path()).expect("uninstall broken managed link");
    assert_eq!(inspect_entry(&link), EntryStatus::NotInstalled);
}

#[cfg(unix)]
#[test]
fn install_then_remove_round_trips_entry_status() {
    let temp = tempfile::tempdir().expect("temp dir");
    let target = temp.path().join("JayJay.AppImage");
    write_executable(&target);
    let bin_dir = temp.path().join("bin");
    let link = bin_dir.join(CLI_NAME);

    assert_eq!(inspect_entry(&link), EntryStatus::NotInstalled);
    let installed = install_symlink(&bin_dir, &target).expect("install");
    assert_eq!(installed, link);
    assert_eq!(inspect_entry(&link), EntryStatus::Installed { target });
    perform_uninstall(&bin_dir).expect("uninstall");
    assert_eq!(inspect_entry(&link), EntryStatus::NotInstalled);
}

#[cfg(unix)]
#[test]
fn install_replaces_broken_symlink() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir");
    let link = bin_dir.join(CLI_NAME);
    let stale = temp.path().join("JayJay-Old.AppImage");
    std::os::unix::fs::symlink(&stale, &link).expect("stale symlink");
    assert_eq!(inspect_entry(&link), EntryStatus::Broken { target: stale });

    let target = temp.path().join("JayJay-New.AppImage");
    write_executable(&target);
    install_symlink(&bin_dir, &target).expect("reinstall over broken link");
    assert_eq!(inspect_entry(&link), EntryStatus::Installed { target });
}

#[cfg(unix)]
#[test]
fn install_never_clobbers_a_foreign_symlink() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir");
    let other = temp.path().join("other-tool");
    write_executable(&other);
    let link = bin_dir.join(CLI_NAME);
    std::os::unix::fs::symlink(&other, &link).expect("symlink");
    let target = temp.path().join("JayJay.AppImage");
    write_executable(&target);

    let result = install_symlink(&bin_dir, &target);
    assert!(result.is_err_and(|err| err.to_string().contains("already exists")));
    assert_eq!(
        fs::read_link(&link).expect("foreign link must survive"),
        other
    );
}

#[cfg(unix)]
#[test]
fn install_never_clobbers_a_regular_file_at_the_entry() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir");
    let existing = bin_dir.join(CLI_NAME);
    write_executable(&existing);
    let target = temp.path().join("JayJay.AppImage");
    write_executable(&target);

    let result = install_symlink(&bin_dir, &target);
    assert!(result.is_err_and(|err| err.to_string().contains("already exists")));
    assert!(
        !fs::symlink_metadata(&existing)
            .expect("entry still present")
            .file_type()
            .is_symlink(),
        "the pre-existing regular file must survive untouched"
    );
}

#[test]
fn install_rejects_missing_target() {
    let temp = tempfile::tempdir().expect("temp dir");
    let result = install_symlink(&temp.path().join("bin"), &temp.path().join("gone"));
    assert!(result.is_err());
}

#[cfg(unix)]
#[test]
fn install_rejects_non_executable_target() {
    let temp = tempfile::tempdir().expect("temp dir");
    let target = temp.path().join("JayJay.AppImage");
    fs::write(&target, "data").expect("write file");

    let result = install_symlink(&temp.path().join("bin"), &target);
    assert!(result.is_err_and(|err| err.to_string().contains("executable")));
}

#[test]
fn install_rejects_linking_the_entry_to_itself() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bin_dir = temp.path().join("bin");
    let result = install_symlink(&bin_dir, &bin_dir.join(CLI_NAME));
    assert!(result.is_err());
}

#[test]
fn uninstall_reports_missing_entry() {
    let temp = tempfile::tempdir().expect("temp dir");
    assert!(perform_uninstall(temp.path()).is_err());
}

#[test]
fn dir_on_path_matches_exact_and_trailing_slash_entries() {
    // join_paths uses the platform separator, so this exercises ';' on Windows and ':' on unix.
    let base = std::env::temp_dir();
    let bin = base.join("jayjay-test-bin");
    let other = base.join("other-bin");
    let joined = |entries: &[PathBuf]| {
        std::env::join_paths(entries.iter().cloned())
            .expect("join paths")
            .into_string()
            .expect("utf8 path")
    };
    let trailing = PathBuf::from(format!("{}{}", bin.display(), std::path::MAIN_SEPARATOR));

    assert!(dir_on_path(&bin, &joined(&[other.clone(), bin.clone()])));
    assert!(dir_on_path(&bin, &joined(&[trailing, other.clone()])));
    assert!(!dir_on_path(&bin, &joined(&[other])));
    assert!(!dir_on_path(&bin, ""));
}

#[test]
fn path_hint_matches_login_shell_dialect() {
    let bin = Path::new("/home/user/.local/bin");
    assert!(path_hint_for_shell("/usr/bin/fish", bin).contains("fish_add_path"));
    assert!(path_hint_for_shell("/bin/zsh", bin).contains("~/.zshrc"));
    assert!(path_hint_for_shell("/bin/bash", bin).contains("~/.bashrc"));
    assert!(path_hint_for_shell("/bin/bash", bin).contains("/home/user/.local/bin"));
}

#[cfg(unix)]
#[test]
fn foreign_appimage_symlink_is_unmanaged() {
    let temp = tempfile::tempdir().expect("temp dir");
    let other = temp.path().join("Other.AppImage");
    write_executable(&other);
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(&other, &link).expect("symlink");

    assert_eq!(
        inspect_entry(&link),
        EntryStatus::Unmanaged {
            target: Some(other.clone())
        }
    );
    assert!(perform_uninstall(temp.path()).is_err());
    assert_eq!(fs::read_link(&link).expect("link must survive"), other);
}

#[cfg(unix)]
#[test]
fn dangling_foreign_appimage_symlink_is_not_repairable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let link = temp.path().join(CLI_NAME);
    std::os::unix::fs::symlink(temp.path().join("Gone-Other.AppImage"), &link).expect("symlink");

    assert!(matches!(
        inspect_entry(&link),
        EntryStatus::Unmanaged { .. }
    ));
}

#[cfg(unix)]
#[test]
fn dangling_jayjay_named_appimage_stays_repairable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let link = temp.path().join(CLI_NAME);
    let old = temp.path().join("jayjay-gpui-x86_64-linux.AppImage");
    std::os::unix::fs::symlink(&old, &link).expect("symlink");

    assert_eq!(inspect_entry(&link), EntryStatus::Broken { target: old });
}
