import AppKit

/// What a take must undo on any exit, Ctrl-C included: modifiers its key presses may have left down, the clipboard it pasted through, and the app it launched.
final class TakeCleanup: @unchecked Sendable {
    static let shared = TakeCleanup()

    private let lock = NSLock()
    private var clipboard: PasteboardSnapshot?
    private var pastedChangeCount: Int?
    private var postedKeys = false
    private var app: pid_t?

    func launched(_ pid: pid_t?) {
        lock.withLock { app = pid }
    }

    func willPostKeys() {
        lock.withLock { postedKeys = true }
    }

    func putOnPasteboard(_ text: String) {
        lock.withLock {
            if clipboard == nil {
                clipboard = PasteboardSnapshot.current()
            }
            NSPasteboard.general.clearContents()
            NSPasteboard.general.setString(text, forType: .string)
            pastedChangeCount = NSPasteboard.general.changeCount
        }
    }

    func run(quittingApp: Bool) {
        lock.withLock {
            if postedKeys {
                KeyCombo.releaseModifiers()
            }
            if NSPasteboard.general.changeCount == pastedChangeCount {
                clipboard?.restore()
            }
            if quittingApp, let app {
                NSRunningApplication(processIdentifier: app)?.terminate()
            }
            clipboard = nil
            pastedChangeCount = nil
            postedKeys = false
            app = nil
        }
    }
}
