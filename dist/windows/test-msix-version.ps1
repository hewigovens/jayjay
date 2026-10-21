$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
. "$PSScriptRoot/msix-version.ps1"
foreach ($case in @(
    @('0.3.21', '1.3.21.0'),
    @('0.3.21-rc.1+build.42', '1.3.21.0'),
    @('1.0.0', '2.0.0.0'),
    @('65534.65535.65535', '65535.65535.65535.0')
)) {
    if ((Get-MsixVersion $case[0]) -cne $case[1]) { throw "Incorrect MSIX version for $($case[0])" }
}
foreach ($version in @('65535.0.0', '0.65536.0', '0.0.65536', '0.3', '0.3.21garbage', '0.3.21.1', '-1.0.0')) {
    $rejected = $false
    try { Get-MsixVersion $version | Out-Null } catch { $rejected = $true }
    if (-not $rejected) { throw "Accepted invalid MSIX app version: $version" }
}
Write-Host 'PASS: MSIX version mapping, prereleases, metadata, rollover and Store limits'
