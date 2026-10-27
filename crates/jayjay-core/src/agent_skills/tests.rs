use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;

use super::AgentSkillInstaller;
use super::installer::managed_file;

#[test]
fn installs_repairs_updates_and_removes_the_bundled_skill() {
    let home = tempfile::tempdir().unwrap();
    let installer = AgentSkillInstaller::new(home.path().to_owned());
    let shared = home.path().join(".agents/skills/jayjay");
    let claude = home.path().join(".claude/skills/jayjay");
    assert!(!installer.status().unwrap().installed);
    installer.set_installed(true).unwrap();
    let installed = fs::read(shared.join("SKILL.md")).unwrap();
    assert_eq!(installed, fs::read(claude.join("SKILL.md")).unwrap());
    assert!(installer.status().unwrap().complete);

    let old = managed_file("---\nname: jayjay\n---\nOld bundled instructions\n");
    fs::write(shared.join("SKILL.md"), old).unwrap();
    fs::remove_dir_all(&claude).unwrap();
    let status = installer.status().unwrap();
    assert!(status.installed && !status.complete);
    installer.set_installed(true).unwrap();
    assert_eq!(installed, fs::read(shared.join("SKILL.md")).unwrap());
    assert_eq!(installed, fs::read(claude.join("SKILL.md")).unwrap());

    fs::write(shared.join("notes.txt"), "Keep this").unwrap();
    installer.set_installed(false).unwrap();
    installer.set_installed(false).unwrap();
    assert!(!installer.status().unwrap().installed);
    assert!(!shared.join("SKILL.md").exists());
    assert!(shared.join("notes.txt").exists());
    assert!(!claude.exists());
}

#[test]
fn preserves_custom_and_edited_skills_without_a_partial_install() {
    let home = tempfile::tempdir().unwrap();
    let installer = AgentSkillInstaller::new(home.path().to_owned());
    let shared = home.path().join(".agents/skills/jayjay");
    let claude = home.path().join(".claude/skills/jayjay");
    installer.set_installed(true).unwrap();
    let edited = fs::read_to_string(shared.join("SKILL.md"))
        .unwrap()
        .replace("# JayJay", "# My JayJay");
    fs::write(shared.join("SKILL.md"), &edited).unwrap();
    fs::remove_dir_all(&claude).unwrap();
    let error = installer.set_installed(true).unwrap_err();
    assert!(error.to_string().contains(shared.to_str().unwrap()));
    assert!(!claude.exists());

    fs::create_dir(&claude).unwrap();
    fs::write(claude.join("SKILL.md"), "Custom instructions").unwrap();
    installer.set_installed(false).unwrap();
    assert_eq!(fs::read_to_string(shared.join("SKILL.md")).unwrap(), edited);
    assert_eq!(
        fs::read_to_string(claude.join("SKILL.md")).unwrap(),
        "Custom instructions"
    );
}

#[cfg(unix)]
#[test]
fn preserves_symlinked_skills_and_dangling_links() {
    for directory_link in [false, true] {
        let home = tempfile::tempdir().unwrap();
        let installer = AgentSkillInstaller::new(home.path().to_owned());
        let skill = home.path().join(".claude/skills/jayjay");
        fs::create_dir_all(skill.parent().unwrap()).unwrap();
        let target = home.path().join("missing");
        let link = if directory_link {
            skill
        } else {
            fs::create_dir(&skill).unwrap();
            skill.join("SKILL.md")
        };
        symlink(&target, &link).unwrap();
        assert!(installer.set_installed(true).is_err());
        installer.set_installed(false).unwrap();
        assert_eq!(fs::read_link(&link).unwrap(), target);
        assert!(!target.exists());
    }
}

#[cfg(unix)]
#[test]
fn shared_discovery_directories_can_point_to_the_same_location() {
    let home = tempfile::tempdir().unwrap();
    let installer = AgentSkillInstaller::new(home.path().to_owned());
    let shared = home.path().join(".agents/skills");
    let claude = home.path().join(".claude/skills");
    fs::create_dir_all(&shared).unwrap();
    fs::create_dir_all(claude.parent().unwrap()).unwrap();
    symlink(&shared, &claude).unwrap();
    installer.set_installed(true).unwrap();
    assert!(installer.status().unwrap().complete);
    installer.set_installed(false).unwrap();
    assert!(!installer.status().unwrap().installed);
    assert_eq!(fs::read_link(&claude).unwrap(), shared);
}
