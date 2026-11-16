use font_kit::source::SystemSource;

/// font-kit's fontconfig lookup by name panics on a face without an index, so match against the listed family names instead.
pub(crate) fn pick(candidates: &[&str]) -> Option<String> {
    let installed = SystemSource::new().all_families().unwrap_or_default();
    candidates.iter().find_map(|name| {
        installed
            .iter()
            .find(|family| family.eq_ignore_ascii_case(name))
            .cloned()
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process::Command;

    use font_kit::source::SystemSource;

    use super::super::linux_windows::{monospaced_families, system_monospace};
    use super::pick;

    const CHILD: &str = "JAYJAY_INDEXLESS_FONTS_CHILD";

    #[test]
    fn faces_without_an_index_are_skipped() {
        if env::var_os(CHILD).is_some() {
            let family = SystemSource::new().all_families().expect("fonts")[0].clone();
            assert_eq!(pick(&[&family]), Some(family));
            assert_eq!(system_monospace(), None);
            assert!(monospaced_families().is_empty());
            return;
        }
        // FONTCONFIG_FILE is process-wide, so the lookups run in a child test process.
        let dir = tempfile::tempdir().expect("temp dir");
        let config = dir.path().join("fonts.conf");
        fs::write(
            &config,
            format!(
                r#"<fontconfig><dir>/usr/share/fonts</dir><cachedir>{}</cachedir><match target="scan"><edit name="index" mode="delete_all"/></match></fontconfig>"#,
                dir.path().display()
            ),
        )
        .expect("write fonts.conf");
        let output = Command::new(env::current_exe().expect("test binary"))
            .args([
                "--exact",
                "app::fonts::platform::fontconfig::tests::faces_without_an_index_are_skipped",
            ])
            .env(CHILD, "1")
            .env("FONTCONFIG_FILE", &config)
            .output()
            .expect("run child test");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}
