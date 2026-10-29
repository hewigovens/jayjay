param([string]$OutputDirectory = "build/windows", [string]$Version = "")
$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest
if (-not $IsWindows) { throw "Build this package on Windows x64 with the MSVC tools and Windows SDK installed." }
$root = (Resolve-Path "$PSScriptRoot/../..").Path
Push-Location $root
$savedFlags = $env:CARGO_ENCODED_RUSTFLAGS
try {
    if ((rustc --version) -notmatch '^rustc 1\.99\.0 ') { throw "Use Rust 1.99.0 for the portable beta." }
    if ((cargo about --version) -notmatch '0\.9\.2$') { throw "Install cargo-about 0.9.2: cargo install cargo-about --version 0.9.2 --features cli --locked" }
    $metadata = cargo metadata --locked --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE) { throw "Cargo metadata failed" }
    $appVersion = ($metadata.packages | Where-Object name -EQ "jayjay-gpui").version
    if (-not $Version) { $Version = $appVersion }
    # cargo-about's metadata also needs the locked sources for non-Windows targets.
    cargo fetch --locked
    if ($LASTEXITCODE) { throw "Dependency source fetch failed" }
    $env:CARGO_ENCODED_RUSTFLAGS = @('-Ctarget-feature=+crt-static', '-Clink-arg=/Brepro', "--remap-path-prefix=$root=.") -join [char]0x1f
    cargo build --locked --release --target x86_64-pc-windows-msvc -p jayjay-gpui -p jayjay-cli
    if ($LASTEXITCODE) { throw "Release build failed" }
    $payload = Join-Path $root "build/windows-payload"
    if (Test-Path -LiteralPath $payload) { Remove-Item -LiteralPath $payload -Recurse -Force }
    New-Item -ItemType Directory -Path $payload | Out-Null
    $binaries = "target/x86_64-pc-windows-msvc/release"
    Copy-Item "$binaries/jayjay.exe", "$binaries/jayjay-gpui.exe" $payload
    Get-ChildItem $binaries -Filter *.dll | Copy-Item -Destination $payload
    Copy-Item LICENSE "$payload/LICENSE.txt"
    Copy-Item crates/LICENSE "$payload/CORE-LICENSE.txt"
    Copy-Item shell/gpui/assets/fonts/Octicons.LICENSE "$payload/OCTICONS-LICENSE.txt"
    Copy-Item assets/fonts/Seti.LICENSE "$payload/SETI-LICENSE.txt"
    Copy-Item dist/windows/README.txt $payload
    foreach ($item in @(@('shell/gpui/Cargo.toml', 'GUI'), @('crates/jayjay-cli/Cargo.toml', 'CLI'))) {
        cargo about generate --offline --locked --fail --manifest-path $item[0] --config dist/windows/about.toml --output-file "$payload/THIRD-PARTY-$($item[1]).html" dist/windows/licenses.hbs
        if ($LASTEXITCODE) { throw "Dependency license generation failed" }
    }
    python dist/windows/package.py $payload $Version $OutputDirectory
    if ($LASTEXITCODE) { throw "PE validation or ZIP packaging failed" }
    & "$PSScriptRoot/msix.ps1" -Payload $payload -Version $Version -AppVersion $appVersion -OutputDirectory $OutputDirectory
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $savedFlags
    Pop-Location
}
