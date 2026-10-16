param([Parameter(Mandatory)][string]$Archive, [switch]$Desktop)
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
$Archive = (Resolve-Path -LiteralPath $Archive).Path
$expected = ((Get-Content -LiteralPath "$Archive.sha256").Trim() -split '\s+')[0]
if ((Get-FileHash -LiteralPath $Archive -Algorithm SHA256).Hash -ne $expected) { throw "ZIP checksum mismatch" }
$temp = Join-Path ([IO.Path]::GetTempPath()) "jj 名 & %PATH% ' $([IO.Path]::GetRandomFileName())"
$contract = Join-Path ([IO.Path]::GetTempPath()) "jayjay-launch-contract-$([guid]::NewGuid())"
New-Item -ItemType Directory -Path $temp | Out-Null

function Set-IsolatedProfile([Diagnostics.ProcessStartInfo]$Info) {
    foreach ($name in @('USERPROFILE', 'HOME', 'APPDATA', 'LOCALAPPDATA')) { $Info.Environment[$name] = "$temp/profile" }
}

function ConvertTo-HexArgs([string[]]$Values) {
    $Values | ForEach-Object { [Convert]::ToHexString([Text.Encoding]::UTF8.GetBytes($_)).ToLowerInvariant() }
}

# Base64 JSON argv, as the GUI passes to --windows-terminal-tool.
function ConvertTo-ToolPayload([string[]]$Argv) {
    [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes((ConvertTo-Json -Compress -InputObject $Argv)))
}

function Invoke-Checked([string]$Executable, [string[]]$Arguments, [int]$ExitCode = 0, [string]$ArgvFile = '') {
    $info = [Diagnostics.ProcessStartInfo]::new($Executable)
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardOutputEncoding = [Text.Encoding]::UTF8
    $info.StandardErrorEncoding = [Text.Encoding]::UTF8
    foreach ($arg in $Arguments) { $info.ArgumentList.Add($arg) }
    Set-IsolatedProfile $info
    $info.Environment['JJ_CONFIG'] = "$temp/jj.toml"
    if ($ArgvFile) { $info.Environment['JAYJAY_TEST_ARGV_FILE'] = $ArgvFile }
    $process = [Diagnostics.Process]::Start($info)
    $stdout = $process.StandardOutput.ReadToEndAsync()
    $stderr = $process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit(30000)) { $process.Kill($true); throw "Timed out: $Executable $Arguments" }
    $result = @{ Out = $stdout.GetAwaiter().GetResult(); Err = $stderr.GetAwaiter().GetResult(); Code = $process.ExitCode }
    if ($result.Code -ne $ExitCode) { throw "Expected exit $ExitCode, got $($result.Code): $($result.Err)" }
    return $result
}

try {
    $payload = Join-Path $temp 'payload'
    Expand-Archive -LiteralPath $Archive -DestinationPath $payload
    foreach ($line in Get-Content -LiteralPath "$payload/SHA256SUMS") {
        $hash, $name = $line -split '  ', 2
        if ((Get-FileHash -LiteralPath (Join-Path $payload $name) -Algorithm SHA256).Hash -ne $hash) { throw "Payload checksum mismatch: $name" }
    }
    $cli = "$payload/jayjay.exe"
    $gui = "$payload/jayjay-gpui.exe"
    foreach ($exe in @($cli, $gui)) {
        if ((Invoke-Checked $exe @('--version')).Out -notmatch '^jayjay \d+\.\d+\.\d+') { throw "Version output missing: $exe" }
        $help = (Invoke-Checked $exe @('--help')).Out
        if ($help -notmatch 'Usage: jayjay' -or $help -notmatch '--repo') { throw "Help output missing: $exe" }
        if ((Invoke-Checked $exe @('config')).Out -notmatch 'merge-tools') { throw "Config output missing: $exe" }
    }
    New-Item -ItemType Directory -Path "$temp/profile" | Out-Null
    Set-Content -LiteralPath "$temp/jj.toml" -Value "[user]`nname = 'Portable smoke'`nemail = 'smoke@example.com'"
    $repo = "$temp/repo 名 & %PATH% '"
    Invoke-Checked (Get-Command jj.exe).Source @('git', 'init', $repo) | Out-Null
    Set-Content -LiteralPath "$repo/file name.txt" -Value 'portable smoke'
    $status = (Invoke-Checked $cli @('review', 'status', '--repo', $repo, '--format', 'json')).Out | ConvertFrom-Json
    if (-not $status.commit_id -or $status.files.Count -ne 1) { throw 'Review dispatch did not read the extracted CLI repository' }
    if (-not (Invoke-Checked $cli @('review', 'status', '--repo', "$temp/missing") 1).Err) { throw 'Error output missing' }
    if ((Invoke-Checked $gui @('--windows-terminal-tool', (ConvertTo-ToolPayload @($cli, '--version')))).Out -notmatch '^jayjay ') { throw 'Terminal helper output missing' }

    New-Item -ItemType Directory -Path $contract | Out-Null
    Copy-Item -LiteralPath $cli -Destination "$contract/jayjay.exe"
    @'
#![windows_subsystem = "windows"]
fn main() {
    let args = std::env::args().skip(1).map(|arg| arg.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()).collect::<Vec<_>>().join("\n");
    if let Some(path) = std::env::var_os("JAYJAY_TEST_ARGV_FILE") {
        let path = std::path::PathBuf::from(path);
        let pending = path.with_extension("pending");
        std::fs::write(&pending, &args).unwrap();
        std::fs::rename(pending, path).unwrap();
    }
    println!("{args}");
    eprintln!("stderr-preserved");
    std::thread::sleep(std::time::Duration::from_millis(750));
    std::process::exit(37);
}
'@ | Set-Content -LiteralPath "$contract/child.rs"
    rustc --edition=2024 "$contract/child.rs" -o "$contract/jayjay-gpui.exe"
    if ($LASTEXITCODE) { throw 'Could not build launcher contract fixture' }
    $left = "C:\repo 名\left & %PATH% ' \" + '"quoted"'
    $right = 'C:\right\trailing\'
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $result = Invoke-Checked "$contract/jayjay.exe" @($left, $right) 37
    $expectedArgs = ConvertTo-HexArgs @('tool', 'diff', $left, $right)
    if ((($result.Out.Trim() -split '\r?\n') -join ',') -cne ($expectedArgs -join ',')) { throw 'Forwarded argv changed' }
    if ($result.Err.Trim() -ne 'stderr-preserved' -or $timer.ElapsedMilliseconds -lt 700) { throw 'Launcher lost stderr or returned before the tool ended' }
    $argvFile = "$contract/launched-argv.txt"
    $launchArgs = @('--repo', 'C:\missing repository 名 & %PATH%')
    $launch = Invoke-Checked "$contract/jayjay.exe" $launchArgs -ArgvFile $argvFile
    if ($launch.Out -or $launch.Err) { throw 'Ordinary GUI launch retained the caller output pipes' }
    for ($attempt = 0; $attempt -lt 100 -and -not (Test-Path -LiteralPath $argvFile); $attempt++) { Start-Sleep -Milliseconds 50 }
    if (((Get-Content -LiteralPath $argvFile) -join ',') -cne ((ConvertTo-HexArgs $launchArgs) -join ',')) { throw 'Repository launch did not reach the sibling GUI' }
    $wrapper = "$contract/wrapped tool.cmd"
    Set-Content -LiteralPath $wrapper -Value '@echo off', '"%~dp0jayjay-gpui.exe" %*'
    $wrapped = Invoke-Checked $gui @('--windows-terminal-tool', (ConvertTo-ToolPayload @($wrapper, $left, $right))) 37
    if ((($wrapped.Out.Trim() -split '\r?\n') -join ',') -cne (($expectedArgs | Select-Object -Skip 2) -join ',')) { throw 'Batch wrapper changed literal tool arguments' }
    Write-Host 'PASS: extracted CLI help/version/config/review/errors, terminal helper, batch arguments, launcher argv/stdio/wait/exit contract'

    $start = [Diagnostics.ProcessStartInfo]::new($gui)
    $start.UseShellExecute = $false
    $start.RedirectStandardError = $true
    $start.RedirectStandardOutput = $true
    $start.ArgumentList.Add('--repo'); $start.ArgumentList.Add($repo)
    Set-IsolatedProfile $start
    $app = [Diagnostics.Process]::Start($start)
    $null = $app.StandardOutput.ReadToEndAsync()
    $appErr = $app.StandardError.ReadToEndAsync()
    try {
        Start-Sleep -Seconds 10
        if ($app.HasExited) { throw "GUI launch exited early: $($app.ExitCode): $($appErr.GetAwaiter().GetResult())" }
        $app.Refresh()
        if ($Desktop -and $app.MainWindowHandle -eq 0) { throw 'No GUI window observed on the requested desktop' }
        Write-Host "PASS: extracted GUI remained alive for 10 seconds; observed window handle: $($app.MainWindowHandle)"
        Write-Host 'UNVERIFIED: rendering, interaction, scaling, authentication, real editor/terminal desktop sessions and external-tool window completion'
    } finally { if (-not $app.HasExited) { $app.Kill($true); $app.WaitForExit() } }
} finally {
    Remove-Item -LiteralPath $temp -Recurse -Force
    if (Test-Path -LiteralPath $contract) { Remove-Item -LiteralPath $contract -Recurse -Force }
}
