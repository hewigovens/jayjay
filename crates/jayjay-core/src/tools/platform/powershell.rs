use base64::{Engine, engine::general_purpose::STANDARD};

/// PowerShell also closes single-quoted strings at U+2018–U+201B.
pub(super) fn quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('\'');
    for c in value.chars() {
        if matches!(c, '\'' | '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}') {
            quoted.push(c);
        }
        quoted.push(c);
    }
    quoted.push('\'');
    quoted
}

pub(super) fn terminal_payload(cwd: &str, command: Option<&str>) -> String {
    let script = format!(
        "Set-Location -LiteralPath {} -ErrorAction Stop; {}",
        quote(cwd),
        command.unwrap_or("")
    );
    STANDARD.encode(
        script
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_and_argv_stay_literal_in_encoded_powershell() {
        let path = r#"C:\名 repo\a'b $env:PATH & ; `(x) \"quoted\"\"#;
        let line = format!("& 'nvim.exe' {}", quote(path));
        assert_eq!(
            line,
            "& 'nvim.exe' 'C:\\名 repo\\a''b $env:PATH & ; `(x) \\\"quoted\\\"\\'"
        );
        assert_eq!(
            quote("a\u{2019}; calc; \u{2018}"),
            "'a\u{2019}\u{2019}; calc; \u{2018}\u{2018}'"
        );
        let bytes = STANDARD
            .decode(terminal_payload(path, Some(&line)))
            .unwrap();
        let units: Vec<_> = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect();
        assert_eq!(
            String::from_utf16(&units).unwrap(),
            format!(
                "Set-Location -LiteralPath {} -ErrorAction Stop; {line}",
                quote(path)
            )
        );
    }

    #[cfg(windows)]
    #[test]
    fn powershell_executes_literal_arguments_in_the_repository() {
        let temp = tempfile::tempdir().unwrap();
        let cwd = temp.path().join("名 repo & $env - apostrophe's");
        std::fs::create_dir(&cwd).unwrap();
        let argument = r#"a'b ’; exit 9; ‘ "quoted" C:\trailing\ & ; %PATH% $env:PATH `whoami`"#;
        let script = format!(
            "[Console]::OutputEncoding = [Text.UTF8Encoding]::new(); [Console]::WriteLine((Get-Location).Path); Write-Output {} | ConvertTo-Json -Compress",
            quote(argument)
        );
        let payload = terminal_payload(cwd.to_str().unwrap(), Some(&script));
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-EncodedCommand", &payload])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout).unwrap();
        let mut lines = text.lines();
        assert_eq!(
            std::fs::canonicalize(lines.next().unwrap()).unwrap(),
            std::fs::canonicalize(cwd).unwrap()
        );
        assert_eq!(
            serde_json::from_str::<String>(lines.next().unwrap()).unwrap(),
            argument
        );
    }
}
