use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Stdio;

use base64::{Engine, engine::general_purpose::STANDARD};
use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
use windows_sys::Win32::System::Console::{
    ATTACH_PARENT_PROCESS, AttachConsole, GetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
    STD_OUTPUT_HANDLE, SetStdHandle,
};

use super::super::powershell;
use crate::repo::subprocess_command;

const TERMINAL_TOOL_ARGUMENT: &str = "--windows-terminal-tool";

/// Runs the tool through the GUI binary so batch-file escaping stays in Rust.
pub fn terminal_line(argv: &[String]) -> Option<String> {
    terminal_invocation(&std::env::current_exe().ok()?, argv)
}

pub fn run_terminal_tool(arguments: &[String]) -> Option<i32> {
    match arguments {
        [flag, rest @ ..] if flag == TERMINAL_TOOL_ARGUMENT => Some(run(rest)),
        _ => None,
    }
}

fn terminal_invocation(executable: &Path, argv: &[String]) -> Option<String> {
    let payload = STANDARD.encode(serde_json::to_vec(argv).ok()?);
    Some(format!(
        "Start-Process -FilePath {} -ArgumentList '{}','{}' -NoNewWindow -Wait",
        powershell::quote(executable.to_str()?),
        TERMINAL_TOOL_ARGUMENT,
        payload,
    ))
}

fn run(arguments: &[String]) -> i32 {
    let argv = match arguments {
        [payload] => STANDARD
            .decode(payload)
            .ok()
            .and_then(|json| serde_json::from_slice::<Vec<String>>(&json).ok())
            .filter(|argv| !argv.is_empty()),
        _ => None,
    };
    let Some(argv) = argv else {
        eprintln!("error: invalid terminal tool payload");
        return 2;
    };
    // SAFETY: only valid handles captured before attaching are restored.
    unsafe {
        let handles = [STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, STD_ERROR_HANDLE]
            .map(|kind| (kind, GetStdHandle(kind)));
        if AttachConsole(ATTACH_PARENT_PROCESS) != 0 {
            for (kind, handle) in handles {
                if !handle.is_null() && handle != INVALID_HANDLE_VALUE {
                    SetStdHandle(kind, handle);
                }
            }
        }
    }
    match subprocess_command(&argv[0])
        .args(&argv[1..])
        .creation_flags(0)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) => {
            eprintln!("error: failed to run terminal tool: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powershell_waits_for_gui_terminal_helper() {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("child.rs");
        let executable = temp.path().join("helper 名 & apostrophe's.exe");
        let marker = temp.path().join("finished");
        std::fs::write(
            &source,
            r#"#![windows_subsystem = "windows"]
fn main() {
    std::thread::sleep(std::time::Duration::from_millis(750));
    std::fs::write(std::env::var_os("JAYJAY_TEST_MARKER").unwrap(), "done").unwrap();
}"#,
        )
        .unwrap();
        assert!(
            std::process::Command::new("rustc")
                .arg(&source)
                .arg("-o")
                .arg(&executable)
                .status()
                .unwrap()
                .success()
        );
        let script = format!(
            "{}; if (!(Test-Path -LiteralPath {})) {{ exit 19 }}",
            terminal_invocation(&executable, &[]).unwrap(),
            powershell::quote(marker.to_str().unwrap()),
        );
        let payload = powershell::terminal_payload(temp.path().to_str().unwrap(), Some(&script));
        let output = std::process::Command::new("powershell.exe")
            .args(["-NoProfile", "-EncodedCommand", &payload])
            .env("JAYJAY_TEST_MARKER", &marker)
            .output()
            .unwrap();
        for _ in 0..40 {
            if marker.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(marker.exists());
    }
}
