use crate::types::*;

/// Find the jj binary. macOS app bundles don't inherit shell PATH.
pub(crate) fn jj_binary() -> String {
    let candidates = ["/opt/homebrew/bin/jj", "/usr/local/bin/jj", "/usr/bin/jj"];
    if let Ok(home) = std::env::var("HOME") {
        let cargo_jj = format!("{home}/.cargo/bin/jj");
        if std::path::Path::new(&cargo_jj).exists() {
            return cargo_jj;
        }
    }
    for path in candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    "jj".to_string()
}

/// Check if jj is installed and return status info.
pub fn check_jj_environment() -> JJStatus {
    let binary = jj_binary();
    if binary == "jj" {
        match std::process::Command::new("jj").arg("version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return JJStatus {
                    is_installed: true,
                    version,
                    path: "jj".to_string(),
                };
            }
            _ => {
                return JJStatus {
                    is_installed: false,
                    version: String::new(),
                    path: String::new(),
                };
            }
        }
    }
    let version = std::process::Command::new(&binary)
        .arg("version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    JJStatus {
        is_installed: true,
        version,
        path: binary,
    }
}
