import AppKit
import SwiftUI

/// Scenes disable state restoration and with it SwiftUI's frame autosave, so frames are kept per scene here.
struct WindowFramePersistence: NSViewRepresentable {
    let key: String
    var defaults: UserDefaults = .standard

    final class Coordinator {
        var observers: [NSObjectProtocol] = []
        var settling = false

        deinit {
            removeObservers()
        }

        func removeObservers() {
            observers.forEach(NotificationCenter.default.removeObserver)
            observers.removeAll()
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    static func dismantleNSView(_ nsView: NSView, coordinator: Coordinator) {
        coordinator.removeObservers()
    }

    func makeNSView(context: Context) -> NSView {
        let view = WindowAttachmentView()
        let coordinator = context.coordinator
        view.onAttach = { window in
            let restored = WindowFrameStore.restore(window, key: key, defaults: defaults)
            // SwiftUI places the window in the run-loop turn the content attaches in; input and accessibility requests arrive only after the loop has waited.
            coordinator.settling = true
            let idle = CFRunLoopObserverCreateWithHandler(nil, CFRunLoopActivity.beforeWaiting.rawValue, false, .max) { [weak coordinator] _, _ in
                coordinator?.settling = false
            }
            CFRunLoopAddObserver(CFRunLoopGetMain(), idle, .commonModes)
            let center = NotificationCenter.default
            let names = [
                NSWindow.didMoveNotification,
                NSWindow.didResizeNotification,
                NSWindow.didEndLiveResizeNotification,
                NSWindow.willCloseNotification
            ]
            // Weak: the blocks live in NotificationCenter and would keep a closed window alive.
            for name in names {
                coordinator.observers.append(center.addObserver(forName: name, object: window, queue: .main) { [weak window, weak coordinator] notification in
                    guard let window else { return }
                    if coordinator?.settling == true {
                        if let restored, window.frame != restored {
                            window.setFrame(restored, display: false)
                        }
                    } else if !window.inLiveResize {
                        WindowFrameStore.save(window.frame, key: key, defaults: defaults)
                    }
                    if notification.name == NSWindow.willCloseNotification {
                        coordinator?.removeObservers()
                    }
                })
            }
        }
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {}
}

/// Fires before SwiftUI orders the window front, so the frame can still change without a visible jump.
final class WindowAttachmentView: NSView {
    var onAttach: ((NSWindow) -> Void)?

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        guard let window, let onAttach else { return }
        self.onAttach = nil
        onAttach(window)
    }
}

enum WindowFrameStore {
    /// Creating the window at the saved size avoids resizing it once shown.
    static func defaultSize(key: String, fallback: CGSize) -> CGSize {
        frame(key: key)?.size ?? fallback
    }

    static func frame(key: String, defaults: UserDefaults = .standard) -> NSRect? {
        guard let string = defaults.string(forKey: defaultsKey(key)) else { return nil }
        let frame = NSRectFromString(string)
        return frame.isEmpty ? nil : frame
    }

    static func save(_ frame: NSRect, key: String, defaults: UserDefaults = .standard) {
        defaults.set(NSStringFromRect(frame), forKey: defaultsKey(key))
    }

    /// Only the first window of a scene takes the saved frame; the key doubles as the window identifier.
    @MainActor
    @discardableResult
    static func restore(_ window: NSWindow, key: String, defaults: UserDefaults = .standard) -> NSRect? {
        window.identifier = NSUserInterfaceItemIdentifier(key)
        guard let saved = frame(key: key, defaults: defaults),
              !NSApp.windows.contains(where: { $0 !== window && ($0.isVisible || $0.isMiniaturized) && $0.identifier?.rawValue == key })
        else { return nil }
        let target = WindowContentSizer.fittedFrame(saved, within: screen(for: saved, fallback: window.screen).visibleFrame)
        window.setFrame(target, display: false)
        return target
    }

    /// Return the window to the display it was saved on instead of clamping it onto the creating screen.
    private static func screen(for frame: NSRect, fallback: NSScreen?) -> NSScreen {
        let best = NSScreen.screens.max { lhs, rhs in
            area(frame.intersection(lhs.frame)) < area(frame.intersection(rhs.frame))
        }
        if let best, area(frame.intersection(best.frame)) > 0 {
            return best
        }
        return fallback ?? NSScreen.main ?? NSScreen.screens[0]
    }

    private static func area(_ rect: NSRect) -> CGFloat {
        rect.isNull ? 0 : rect.width * rect.height
    }

    private static func defaultsKey(_ key: String) -> String {
        "jayjay.windowFrame.\(key)"
    }
}
