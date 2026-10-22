param([Parameter(Mandatory)][string]$Payload, [Parameter(Mandatory)][string]$Version, [Parameter(Mandatory)][string]$AppVersion, [string]$OutputDirectory = "build/windows")
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if ($AppVersion -notmatch '^(\d+)\.(\d+)\.(\d+)(?:[-+].*)?$') { throw "MSIX needs a major.minor.patch app version, got $AppVersion" }
$major, $minor, $patch = $Matches[1..3] | ForEach-Object { [long]$_ }
if ($major -ge 65535 -or $minor -gt 65535 -or $patch -gt 65535) { throw "App version exceeds the Store MSIX version range: $AppVersion" }
# The Store rejects a zero first section and reserves the fourth; major+1 keeps 0.x and later releases increasing.
$packageVersion = "$($major + 1).$minor.$patch.0"
$sdk = Get-ChildItem "${env:ProgramFiles(x86)}/Windows Kits/10/bin/*/x64/makeappx.exe" |
    Sort-Object { [version]$_.Directory.Parent.Name } | Select-Object -Last 1
if (-not $sdk) { throw "makeappx.exe not found; install the Windows SDK" }
$makeappx = $sdk.FullName
$makepri = Join-Path $sdk.DirectoryName "makepri.exe"

$work = Join-Path (Split-Path (Resolve-Path $Payload).Path) "windows-msix"
$layout = Join-Path $work "layout"
if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
New-Item -ItemType Directory -Path $layout | Out-Null
Copy-Item "$Payload/*" $layout
Copy-Item "$PSScriptRoot/msix/Assets" $layout -Recurse
# Without default and light-theme variants the taskbar plates the icon.
Get-ChildItem "$layout/Assets/*_altform-unplated.png" | ForEach-Object {
    Copy-Item $_.FullName ($_.FullName -replace '_altform-unplated', '')
    Copy-Item $_.FullName ($_.FullName -replace '_altform-unplated', '_altform-lightunplated')
}
[xml]$manifest = Get-Content -LiteralPath "$PSScriptRoot/msix/AppxManifest.xml"
$manifest.Package.Identity.Version = $packageVersion
$manifest.Save((Join-Path $layout "AppxManifest.xml"))

# Without resources.pri the shell ignores the targetsize/unplated logo variants.
& $makepri createconfig /cf "$work/priconfig.xml" /dq en-US /o
if ($LASTEXITCODE) { throw "makepri createconfig failed" }
& $makepri new /pr $layout /cf "$work/priconfig.xml" /mn "$layout/AppxManifest.xml" /of "$layout/resources.pri" /o
if ($LASTEXITCODE) { throw "makepri new failed" }

New-Item -ItemType Directory -Path $OutputDirectory -Force | Out-Null
$package = Join-Path $OutputDirectory "JayJay-$Version-windows-x64.msix"
& $makeappx pack /o /d $layout /p $package
if ($LASTEXITCODE) { throw "makeappx pack failed" }
Write-Host $package
