# Run & Debug Guide

Load this file to launch or drive a JayJay build, or to debug a CI, test-runner, or desktop-automation failure. Test placement and fixtures live in [Testing](testing.md).

## Running the App

- `just run [repo]` builds and launches the SwiftUI debug app; `just shell::gpui-run [repo]` launches the GPUI dev bundle. Both run with the process name `JayJay`, so target one by PID (`pgrep -f jayjay-gpui`), never by name.
- On Linux, normal GPUI launches detach from the terminal. Use `jayjay --foreground [repo]` (or `cargo run -p jayjay-gpui -- --foreground [repo]`) to keep the GUI and its logs in the terminal when debugging. CLI commands and external diff/merge tools remain in the foreground.
- Never send synthetic clicks at screen coordinates while the user is active: a stale layout lands the click in the wrong window and can open a modal that writes to the working copy. Capture a window without focusing it with `screencapture -x -o -l <windowID>`.
- Relaunching kills the process together with any unsaved modal.
- Running instances snapshot the repository on every filesystem event; quit them before divergence cleanup or history rewrites (see [Version Control](version-control.md)).

## Windows Builds From macOS

`just shell::gpui-windows` cross-builds `target/x86_64-pc-windows-msvc/release/jayjay-gpui.exe` with `cargo-xwin`, for checking Windows behavior in a VM; CI's portable ZIP remains the release build. gpui precompiles its DirectX shaders only on a Windows build host, so the recipe keeps debug assertions on, which makes the app compile `shaders.hlsl` and `color_text_raster.hlsl` at startup from the gpui checkout path baked into the binary. Copy those files from `~/.cargo/git/checkouts/zed-*/<rev>/crates/gpui_windows/src/` (the revision in `Cargo.lock`) to the same path under `C:\` in the VM, or the app panics with "Recreating GPU state for DirectWrite: The system cannot find the path specified." Runtime shader compilation adds a few hundred milliseconds whenever a window opens, so use these builds to compare behavior, not absolute startup times. Prefix `JAYJAY_PRERELEASE=1` to get Debug mode on by default.

Over SSH, programs run outside the desktop session, so start GUI runs in the logged-in session with an interactive scheduled task (`schtasks /create /it … /tr <command>` then `schtasks /run`). With Debug mode on, the log under `%APPDATA%\hewig\jayjay\data\logs\jayjay.log` records startup timings and main-thread stalls of 250 ms or more.

## Test Runner Recovery

- Run `just test-app` and `just test-ui` in the foreground to completion, one at a time: one test session owns DerivedData and `testmanagerd`, and a backgrounded or concurrent run wedges both.
- "The test runner hung before establishing connection" after a killed run: check `pgrep -fl xcodebuild` for orphans, `kill -9` `testmanagerd` (launchd respawns it), and rerun.
- "Timed out while enabling automation mode": macOS Automation Mode wants a password from someone at the Mac, and `testmanagerd` caches that approval only for a while; the runner's code signature plays no part. `sudo automationmodetool enable-automationmode-without-authentication` removes the prompt on a development machine. "Failed to activate application (current state: Running Background)": the desktop is in use. In both cases do not retry in a loop; report the scene as unverified and use the CI result.

## Debugging From CI

- Passing UI runs discard attachments. For a failing scene, download the `ui-test-xcresult` artifact: `xcrun xcresulttool get test-results tests` gives the failure message, the exported accessibility hierarchy dump shows which element a query hit, and the screen recording shows what the scene saw.
- The GPUI workflow runs the Rust tests on Linux and Windows; a Windows-only failure is usually a path or ref-name portability issue (see [Testing](testing.md#gpui-tests)).
- A blank or stale Help window has its own diagnosis in [Help Book](help-book.md).
