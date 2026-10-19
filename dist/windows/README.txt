JayJay for Windows x64 — portable alpha

Extract the whole ZIP into a writable folder. Start jayjay-gpui.exe for the GUI. Use .\jayjay.exe from PowerShell for CLI output, or add this folder to PATH:

  .\jayjay.exe --version
  .\jayjay.exe "C:\path to\repository"
  .\jayjay.exe review status --repo "C:\path to\repository" --format json
  .\jayjay.exe config

The GUI has no console window. The console launcher prints help/config/review output and preserves exit status. External diff/edit/merge tool sessions block until their window closes. Keep both executables in the same directory. Use the output of "jayjay config" in your jj user config to configure tools.

Requires x64 Windows 10 (version 1903 or later) or Windows 11, with a working Direct3D 11 graphics driver. Rust's CRT is linked statically; Windows supplies the system graphics and shell DLLs. Install jj separately, for example with: winget install jj-vcs.jj (the alpha does not bundle jj; any jj.exe on PATH works). Git and optional gh/glab/Codex/Claude tools must also be installed separately for the features that use them. Authentication remains owned by those tools.

Settings offers Windows editors and terminals. System Editor uses VISUAL or EDITOR when set, otherwise the Windows file association. Windows Terminal runs PowerShell, falling back to Windows PowerShell if wt.exe is unavailable. A custom terminal command must accept an executable and its arguments appended to the configured command. Custom editor/terminal commands use Windows double-quote/backslash argument rules; quote executable paths containing spaces, for example:
  "C:\Program Files\Microsoft VS Code\bin\code.cmd" --wait

Configuration and history use your Windows user application-data directories; portable describes deployment, not a separate data profile. No installation or CLI installer is required. There are no automatic updates or notifications. This alpha is unsigned, so Windows may show a trust/SmartScreen warning.

SHA256SUMS verifies the extracted files; the adjacent .zip.sha256 verifies the archive. LICENSE.txt covers the app; CORE-LICENSE.txt and THIRD-PARTY-*.html contain core and dependency terms. OCTICONS-LICENSE.txt and SETI-LICENSE.txt cover the embedded icon fonts.

Desktop interaction, rendering, DPI/scaling, editor/terminal integration and authentication need testing on an actual Windows desktop. Automated build, headless CLI and launch checks alone do not establish those behaviors.
