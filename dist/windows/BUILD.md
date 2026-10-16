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

Outputs are `build/windows/JayJay-<version>-windows-x64-alpha.zip` and its `.zip.sha256`. The extracted directory includes a `SHA256SUMS` payload manifest. Use `-Version <version-or-build-id>` and `-OutputDirectory <directory>` to label a local candidate. This does not bump the application version.

The ZIP is byte-for-byte deterministic for the same payload: entries are sorted, permissions and timestamps are fixed, and ZIP storage avoids compression-library variation. Reproducing the compiled binaries across different Windows SDKs, checkout locations, or build machines has not been verified.

`.github/workflows/windows-portable.yml` builds, tests, extracts, launches, and retains a CI candidate even if a smoke check fails; check the run result before using it. It does not publish release assets. Existing GPUI CI runs the full Windows Rust workspace tests. The ZIP and its `.zip.sha256` are uploaded unarchived, so each artifact is the file itself. For a smoke-only rerun, dispatch the workflow with `candidate_run_id` set to a prior run that uploaded them. It downloads that exact candidate and checks it with the selected revision's smoke script; it does not build new binaries or rerun the Rust tests.

The smoke check exercises real extracted CLI help/version/config/review/error behavior and the terminal helper. A separate GUI-subsystem fixture verifies the console launcher's literal argument forwarding, output pipes, blocking wait, and exit status without requiring a desktop. It also checks batch-wrapper argument handling through the real terminal helper. Completing real diff/edit/merge windows, File Explorer reveal, default associations, editor and terminal desktop interaction, rendering, scaling, keyboard workflows, and authentication still require a Windows desktop test pass.
