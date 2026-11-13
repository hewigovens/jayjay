use crate::ssh::connection;
use crate::ssh::helper::Helper;
use crate::tools::powershell;
use crate::{JayError, JayResult};
use base64::Engine;

/// `$digest` is set before these, and `$upload` before the finish.
const PRELUDE: &str = r#"
$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Console]::OutputEncoding = [Text.Encoding]::UTF8
$dir = Join-Path $env:LOCALAPPDATA "JayJay\helpers\$digest"
$exe = Join-Path $dir 'jayjay-ssh.exe'
function Test-Helper($path) { (Test-Path -LiteralPath $path) -and ((Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash -eq $digest) }
"#;

const CHECK: &str = r#"
if (Test-Helper $exe) {
  try { (Get-Item -LiteralPath $dir).LastWriteTime = Get-Date } catch {}
  [Console]::Out.Write($exe)
} else {
  New-Item -ItemType Directory -Force -Path $dir | Out-Null
  [Console]::Out.Write('upload:' + (Join-Path $dir "upload.$PID.tmp"))
}
"#;

const FINISH: &str = r#"
try {
  if (-not (Test-Helper $upload)) { throw 'The uploaded SSH helper does not match.' }
  Move-Item -Force -LiteralPath $upload -Destination $exe
} finally {
  Remove-Item -Force -LiteralPath $upload -ErrorAction SilentlyContinue
}
Get-ChildItem -LiteralPath (Split-Path $dir) -Directory | Where-Object { $_.Name -ne $digest -and $_.LastWriteTime -lt (Get-Date).AddDays(-1) } | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
[Console]::Out.Write($exe)
"#;

impl Helper {
    /// cmd and PowerShell login shells can both start PowerShell, but it can't read a large upload from stdin, hence scp.
    pub(crate) fn deploy_windows(&self, machine: &str) -> JayResult<String> {
        let powershell = |body: &str| {
            let script = format!("$digest = '{}'{PRELUDE}{body}", self.hash);
            let utf16: Vec<u8> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
            format!(
                "powershell -NoProfile -NonInteractive -EncodedCommand {}",
                base64::engine::general_purpose::STANDARD.encode(utf16)
            )
        };
        let check = connection::run(machine, &powershell(CHECK), b"")?;
        // cmd and PowerShell mangle a quoted path holding any of these, so the helper would not start.
        let check = check.trim();
        if check.is_empty()
            || check.contains([
                '"', '&', '(', ')', '@', '^', '|', '<', '>', '%', '$', '`', '\u{2018}', '\u{2019}',
                '\0', '\r', '\n',
            ])
        {
            return Err(JayError::internal(format!(
                "The SSH helper can't be started from {check}."
            )));
        }
        let path = match check.strip_prefix("upload:") {
            Some(upload) => {
                connection::upload(
                    machine,
                    &self.path,
                    &crate::ssh::host_path::wire_form(upload),
                )?;
                connection::run(
                    machine,
                    &powershell(&format!("$upload = {}{FINISH}", powershell::quote(upload))),
                    b"",
                )?
                .trim()
                .to_owned()
            }
            None => check.to_owned(),
        };
        Ok(format!("cmd /c \"{path}\""))
    }
}
