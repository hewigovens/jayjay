# Help Book Guide

Load this file before changing the bundled macOS Help Book, the public user guide, help screenshots, command-palette help entries, or Help menu behavior.

## Source Layout

- `docs/guide.html` and `docs/guide.css` are the public web guide.
- `shell/mac/Resources/JayJay.help` is the bundled macOS Help Book.
- Shared Help styling starts in `docs/help-common.css`.
- Help Book-specific styling starts in `docs/help-book.css`.
- `scripts/build-help-book.sh` copies the shared CSS and Help Book CSS into the Help Book, concatenates them into `sty/help.css`, copies `docs/help.js`, converts `docs/imgs/*.webp` screenshots to Help Book PNGs, updates the Help Book version checksum, and rebuilds `JayJay.helpindex` with `hiutil`.
- Use `just shell::help` to rebuild only the Help Book, and `just run` to rebuild the app, reset JayJay Help caches, and launch the debug app.

## Apple Help Rules

- Keep Help pages conservative: XHTML-style markup, relative paths, one `sty/help.css` stylesheet link, and PNG images inside the Help Book.
- WebP screenshots are fine as source assets under `docs/imgs`, but the Help Book must use PNG output from `scripts/build-help-book.sh`.
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
- `just shell::reset-help-cache` kills `Tips`/`helpd` and removes JayJay Help caches. `just run` depends on it.
- A stale or corrupted cache can show old screenshots or old CSS even when the app bundle is correct.
- A blank Help window is not always cache. Check system logs for sandbox denials or read failures:

```bash
/usr/bin/log show --last 5m --style compact --predicate 'eventMessage CONTAINS "JayJay.help" OR eventMessage CONTAINS "dev.hewig.jayjay"'
```

## Verification

Before calling Help Book work done:

1. Run `just shell::help`.
2. Run `just run`.
3. Open `Help > JayJay Help` from the running app.
4. Verify the sidebar, dark/light screenshots, and topic links in the system Help window, not only in a browser.
5. If the Help window is blank, first confirm Swift registers `JayJay.help` directly, then inspect logs before changing cache logic.
