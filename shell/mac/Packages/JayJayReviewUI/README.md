# JayJayReviewUI

SwiftUI review for iOS and macOS hosts that get their review from a server, the native counterpart of
`@jayjay/review-web` ([web/review/README.md](../../../../web/review/README.md)). It draws with JayJayCore's
palettes and catalog strings, so a host links the portable library but none of the repository operations.

- `ReviewBackend` is review-web's backend contract: `open`, `render` and `mutate`, returning `ReviewResult` JSON in
  review-web's shape once a write is durable. Control Room's ctrld serves it (`/api/jayjay`, `/api/jayjay/file`,
  `/api/jayjay/mutate`).
- `ReviewModel` holds one change: its files, the selected file's lines, and mark and note edits. A reply is applied
  only while it is the latest for what it answers, note saves go one at a time, and `open` does nothing while a save
  is in flight. Failures go to `failed`; the host words them.
- Views: `UnifiedDiffReader` (a phone-sized unified diff with word changes and syntax), `ReviewFileLabel`,
  `FileKindBadge`, `ReviewStateIcon` and `ReviewMarkButton`. Set `\.diffThemes` to follow the host's theme, and
  `Localized.setLanguage` to follow its language.

`just shell::test` runs `swift test` here against the app's XCFramework. For another app,
`just shell::ffi-portable <out>` writes JayJayCore (macOS and iOS slices of the portable library), JayJayDiffUI and
this package into `<out>`; `just shell::test-portable` builds that and compiles this package for the iOS Simulator.
