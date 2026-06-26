# Help Book Guide

Load this file before changing the bundled macOS Help Book, the public user guide, help screenshots, command-palette help entries, or Help menu behavior.

## Source Layout

- `docs/guide.html` and `docs/guide.css` are the public web guide.
- `shell/mac/Resources/JayJayHelpBook` is the editable Help Book source. It intentionally does not end in `.help`, because Spotlight/Help Services can discover source `.help` bundles and make Tips try to read the workspace path, which is sandbox-denied.
- `build/help.noindex/JayJay.help` is the generated macOS Help Book that Xcode embeds into the app. The `.noindex` parent keeps Spotlight from treating the generated build artifact as another candidate Help Book.
- Shared Help styling starts in `docs/help-common.css`.
- Help Book-specific styling starts in `docs/help-book.css`.
- `scripts/build-help-book.sh` copies `shell/mac/Resources/JayJayHelpBook` to `build/help.noindex/JayJay.help`, concatenates the shared CSS and Help Book CSS into `sty/help.css`, copies `docs/help.js`, converts source screenshots to downscaled Help Book JPEGs, updates the Help Book version checksum, and rebuilds `JayJay.helpindex` with `hiutil`.
- Use `just shell::help` to rebuild only the Help Book, and `just run` to rebuild the app, reset JayJay Help caches, and launch the debug app.

## Apple Help Rules

- Keep Help pages conservative: simple HTML, relative paths, one `sty/help.css` stylesheet link, and JPEG screenshots inside the Help Book. Do not use an XHTML public doctype with an external DTD URL; Tips' sandbox can treat the DTD fetch as an out-of-book navigation and leave the page blank.
- WebP screenshots are fine as source assets under `docs/imgs`, and `docs/imgs/home.png` is the home source; the Help Book uses JPEG output from `scripts/build-help-book.sh`.
- Keep `CFBundleHelpBookName`, `HPDBookTitle`, and the Swift `HelpBook.bookTitle` string in sync.
- Any content or asset change must change the Help Book `CFBundleVersion`; the build script appends a content checksum for this.
- Do not edit generated copied CSS in the Help Book as the only source. Edit `docs/help-common.css` or `docs/help-book.css`, then run the build script.

## Opening System Help

Register the embedded Help Book bundle, not the containing app bundle:

```swift
guard let helpBookURL = Bundle.main.url(forResource: "JayJay", withExtension: "help") else {
    return false
}
let registerStatus = AHRegisterHelpBookWithURL(helpBookURL as CFURL)
let gotoStatus = AHGotoPage("JayJay Manual" as CFString, relativePath as CFString, nil)
```

Do not pass `Bundle.main.bundleURL` to `AHRegisterHelpBookWithURL`. On macOS 26, the Help UI is hosted by `Tips.app` with bundle id `com.apple.helpviewer`. Registering the app bundle can return `noErr` while the Help window renders blank from a debug app in DerivedData because the Help host cannot read the app-bundle Help Book path.

## Cache And Debugging

- JayJay Help cache commonly lives under `~/Library/Group Containers/group.com.apple.helpviewer.content/Library/Caches/dev.hewig.jayjay.dev.hewig.jayjay.manual*<version>.help`.
- `just shell::reset-help-cache` kills `Tips`/`tipsd`/`helpd`, removes JayJay Help Viewer caches, clears `helpd` cache databases, and removes JayJay-only registered-book entries from `~/Library/Preferences/com.apple.help.plist`. `just run` depends on it.
- `just shell::debug-help` prints source-vs-generated-vs-embedded Help Book versions, image formats, cache paths, Spotlight Help Book records, registered Help books, and recent `Tips`/`helpd` logs. Run it after opening Help when the Help window is blank or stale.
- A stale or corrupted cache can show old screenshots or old CSS even when the app bundle is correct.
- A blank Help window is not always cache. Check system logs for sandbox denials or read failures:

```bash
/usr/bin/log show --last 5m --style compact --predicate 'eventMessage CONTAINS "JayJay.help" OR eventMessage CONTAINS "dev.hewig.jayjay"'
```

When debugging Help Viewer:

1. Run `just shell::build` and confirm the generated Help Book and embedded app bundle have the same Help Book `CFBundleVersion`.
2. Run `just shell::reset-help-cache`, then confirm `find "$HOME/Library/Group Containers/group.com.apple.helpviewer.content/Library/Caches" -maxdepth 2 -iname '*jayjay*' -print` is empty and `just shell::debug-help` reports no JayJay registered-book records before Help is opened again.
3. Launch the built app and open Help from JayJay's Help menu. Do not inspect only the source HTML in a browser; Help Viewer reads the embedded Help Book.
4. If it is stale or blank, run `just shell::debug-help` immediately before rebuilding again. Compare source and embedded Help Book versions, check for non-JPEG screenshots, and read the recent `Tips`/`helpd` log section.
5. If `AHRegisterHelpBookWithURL` and `AHGotoPage` both return `noErr` but Help is blank, suspect Help Viewer sandbox/cache behavior first; verify Swift is registering `Bundle.main.url(forResource: "JayJay", withExtension: "help")`, not `Bundle.main.bundleURL`.
6. If logs mention `Sandbox: Tips deny file-read-data .../shell/mac/Resources/JayJay.help` or `outside the sandbox`, a source `.help` bundle has been reintroduced or `helpd` is still holding a stale registered-book record. The repo should only contain `Resources/JayJayHelpBook`; run `just shell::reset-help-cache`, rebuild, and confirm `mdfind 'kMDItemCFBundleIdentifier == "dev.hewig.jayjay.manual"'` does not point at the workspace source tree and `just shell::debug-help` does not show JayJay entries in `com.apple.help.plist`.

## Verification

Before calling Help Book work done:

1. Run `just shell::help`.
2. Run `just run`.
3. Open `Help > JayJay Help` from the running app.
4. Verify the sidebar, dark/light screenshots, and topic links in the system Help window, not only in a browser.
5. If the Help window is blank, first confirm Swift registers `JayJay.help` directly, then inspect logs before changing cache logic.
