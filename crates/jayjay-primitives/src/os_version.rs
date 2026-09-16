pub fn os_version() -> String {
    detected_os_version().unwrap_or_else(|| "unknown".to_owned())
}

fn detected_os_version() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
            .filter(|version| !version.is_empty())
    }
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/etc/os-release")
            .ok()?
            .lines()
            .find_map(|line| line.strip_prefix("VERSION_ID="))
            .map(|value| value.trim_matches('"').to_owned())
            .filter(|version| !version.is_empty())
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    #[test]
    fn reports_the_product_version() {
        let version = super::os_version();
        assert!(
            version.split('.').all(|part| part.parse::<u32>().is_ok()),
            "{version}"
        );
    }
}
