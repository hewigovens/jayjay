# Windows x64 portable alpha

Build on Windows x64 with Visual Studio C++ build tools, a Windows SDK including `fxc.exe`, Python 3.9 or later, and PowerShell 7. Rust dependencies are locked in `Cargo.lock`; the packaging toolchain is Rust 1.99.0 and cargo-about 0.9.2.

```powershell
rustup toolchain install 1.99.0 --profile minimal
$env:RUSTUP_TOOLCHAIN = '1.99.0'
cargo install cargo-about --version 0.9.2 --features cli --locked
./dist/windows/build.ps1
./dist/windows/smoke.ps1 -Archive (Get-ChildItem build/windows/*.zip).FullName
```

Install jj on PATH before the smoke check. Add `-Desktop` to require an observed window when running on an interactive desktop. The normal smoke check starts the extracted GUI and requires it to stay alive for ten seconds. It reports the window handle without claiming rendering or interaction coverage. A headless runner with no usable Direct3D adapter may fail this launch check; inspect the runner before treating that failure as desktop evidence.

`build.ps1` builds both binaries for `x86_64-pc-windows-msvc`, statically links the CRT, enables MSVC reproducible linking, and embeds the GPUI release shaders. It fetches all locked dependency sources before compilation, collects license reports offline, and copies the application, core, and font licenses. `package.py` verifies x64 PE headers, GUI/console subsystems, and ordinary and delayed DLL imports. Windows system DLLs come from the OS; any other imported DLL must be in the payload. Windows supplies the [ICU DLLs](https://learn.microsoft.com/en-us/windows/win32/intl/international-components-for-unicode--icu-); the alpha requires Windows 10 version 1903 or later, or Windows 11. No jj, Git, authentication tool, installer, or updater is bundled.

Outputs are `build/windows/JayJay-<version>-windows-x64-alpha.zip`, its `.zip.sha256`, and an unsigned `JayJay-<version>-windows-x64.msix`. The MSIX version is `<major+1>.minor.patch.0` from the `jayjay-gpui` crate version (the Store rejects a zero first section), so `0.3.21` packages as `1.3.21.0`, and prereleases and untagged CI builds share the next stable version; submit only stable release builds to the Store, which requires each submission to have a higher version. The extracted directory includes a `SHA256SUMS` payload manifest. Use `-Version <version-or-build-id>` and `-OutputDirectory <directory>` to label a local candidate. This does not bump the application version.

The ZIP is byte-for-byte deterministic for the same payload and zlib version: entries are sorted, permissions and timestamps are fixed, and entries use deflate at level 9. Reproducing the compiled binaries across different Windows SDKs, checkout locations, or build machines has not been verified.

`.github/workflows/windows-portable.yml` builds the ZIP and smoke-checks the extracted payload; GPUI CI runs the Windows Rust workspace tests. Every run uploads the unsigned MSIX as the `windows-msix` artifact for 14 days; CI keeps no other build artifacts. A published `v*` GitHub release hands the ZIP to the `publish` job, which attaches it and its `.zip.sha256` to the release. For stable releases it also copies both to `https://pkgs.hewig.dev/windows/` (R2, never overwritten) and uploads a `winget-manifests` artifact generated from the served bytes; submit that `manifests/h/hewig/JayJay/<version>` directory to microsoft/winget-pkgs. The job reuses the pacman publisher's `R2_*` secrets and runs from `main`.

The smoke check exercises real extracted CLI help/version/config/review/error behavior and the terminal helper. A separate GUI-subsystem fixture verifies the console launcher's literal argument forwarding, output pipes, blocking wait, and exit status without requiring a desktop. It also checks batch-wrapper argument handling through the real terminal helper. Completing real diff/edit/merge windows, File Explorer reveal, default associations, editor and terminal desktop interaction, rendering, scaling, keyboard workflows, and authentication still require a Windows desktop test pass.
