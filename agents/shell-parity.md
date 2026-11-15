# Platform Differences and Remaining Gaps

Use this document when comparing the SwiftUI macOS app with GPUI on Linux and Windows. GPUI on macOS is a development build, not a product parity target. Shared workflows belong in the [user guide](../docs/guide.html); this document records only intentional differences and known gaps.

Refresh it during the [release documentation pass](release.md), or when explicitly asked. Feature PRs should identify changes for that pass without editing this document. Before changing shell code, load the [SwiftUI](swiftui.md) or [GPUI](gpui.md) guide.

## Intentional Platform Differences

| Area | Difference |
| --- | --- |
| AI providers | SwiftUI supports Apple Intelligence alongside Codex and Claude. GPUI on Linux and Windows uses Codex and Claude. |
| HTML previews | SwiftUI offers an inline sandboxed HTML preview. GPUI on Linux and Windows opens HTML externally by design; see [format projections](format-projections.md). |
| Help and feedback | macOS includes a bundled Help Book and Apple Mail integration for attaching diagnostic logs. GPUI on Linux and Windows opens the online guide and reveals logs for attachment. |
| Desktop integration | SwiftUI provides macOS Dock icons and menus, Rex integration, and the Coding Agents skill-install toggle. GPUI follows Linux and Windows desktop conventions; these macOS integrations are not parity gaps. |
| Tools and shortcuts | Linux uses desktop openers and system editor/terminal preferences; Windows uses Windows shell integration. Shortcuts use platform modifiers and native text-editing conventions. The Linux CLI installer links into `~/.local/bin`. |
| Distribution | macOS uses the signed app and Sparkle updater. Linux ships AppImage and Debian packages. Windows ships Store packages and x64/ARM64 portable ZIPs; portable ZIPs have no automatic updater. |

## Remaining Gaps

| Platform | Missing behavior | Evidence |
| --- | --- | --- |
| Windows (GPUI) | Desktop notification when a background fetch or push finishes in an inactive window. Linux implements this notification; Windows currently returns without sending one. | [Windows adapter](../shell/gpui/src/platform/windows.rs), [Linux adapter](../shell/gpui/src/platform/linux.rs) |

This is a list of known gaps, not a claim that every other interaction has been verified on both platforms. Add a gap only after checking current source and tests; identify the affected platform and concrete missing behavior. Remove it when the behavior is implemented and verified. Unverified platform behavior belongs in the validation report, not in this list as an assumed defect.

## Shared Behavior and Verification

- Keep shared domain behavior in Rust; SwiftUI reaches it through UniFFI and GPUI links it directly. Preserve shared configuration, command identities, review state, and jj action semantics.
- Preserve native presentation. Equivalent workflows need not have identical layouts, animations, or menu placement.
- Validate at the smallest useful layer. Linux- or Windows-specific behavior needs evidence from that platform; a GPUI build on macOS proves neither.
- Test behavior rather than constants or wiring. Keep release maturity and installation instructions in the user guide instead of duplicating them here.
