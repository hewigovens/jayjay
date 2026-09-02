# Run & Debug Guide

Load this file to launch or drive a JayJay build, or to debug a CI, test-runner, or desktop-automation failure. Test placement and fixtures live in [Testing](testing.md).

## Running the App

- `just run [repo]` builds and launches the SwiftUI debug app; `just shell::gpui-run [repo]` launches the GPUI dev bundle. Both run with the process name `JayJay`, so target one by PID (`pgrep -f jayjay-gpui`), never by name.
- Never send synthetic clicks at screen coordinates while the user is active: a stale layout lands the click in the wrong window and can open a modal that writes to the working copy. Capture a window without focusing it with `screencapture -x -o -l <windowID>`.
- Relaunching kills the process together with any unsaved modal.
- Running instances snapshot the repository on every filesystem event; quit them before divergence cleanup or history rewrites (see [Version Control](version-control.md)).

## Test Runner Recovery

- Run `just test-app` and `just test-ui` in the foreground to completion, one at a time: one test session owns DerivedData and `testmanagerd`, and a backgrounded or concurrent run wedges both.
- "The test runner hung before establishing connection" after a killed run: check `pgrep -fl xcodebuild` for orphans, `kill -9` `testmanagerd` (launchd respawns it), and rerun.
- "Timed out while enabling automation mode" or "Failed to activate application (current state: Running Background)": the desktop is in use or automation permission is missing. Do not retry in a loop; report the scene as unverified and use the CI result.

## Debugging From CI

- Passing UI runs discard attachments. For a failing scene, download the `ui-test-xcresult` artifact: `xcrun xcresulttool get test-results tests` gives the failure message, the exported accessibility hierarchy dump shows which element a query hit, and the screen recording shows what the scene saw.
- The GPUI workflow runs the Rust tests on Linux and Windows; a Windows-only failure is usually a path or ref-name portability issue (see [Testing](testing.md#gpui-tests)).
- A blank or stale Help window has its own diagnosis in [Help Book](help-book.md).
