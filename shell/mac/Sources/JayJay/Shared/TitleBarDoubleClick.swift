import AppKit
import SwiftUI

/// Runs the title bar's double-click action over the empty toolbar: with the toolbar background hidden, the gaps between items hit-test to the window's content, which ignores it.
struct TitleBarDoubleClick: NSViewRepresentable {
    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }

    func updateNSView(_: NSView, context _: Context) {}

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator {
        private var monitor: Any?

        func install(on view: NSView) {
            monitor = NSEvent.addLocalMonitorForEvents(matching: .leftMouseDown) { [weak view] event in
                guard event.clickCount == 2,
                      let window = view?.window,
                      event.window === window,
                      window.isEmptyTitleBar(at: event.locationInWindow)
                else {
                    return event
                }
                window.performTitleBarDoubleClick()
                return nil
            }
        }

        deinit {
            if let monitor {
                NSEvent.removeMonitor(monitor)
            }
        }
    }
}

private extension NSWindow {
    /// Anything inside a toolbar item's hosting view may be one of its controls, so only the title bar's own plain views count besides the content.
    func isEmptyTitleBar(at point: NSPoint) -> Bool {
        guard !styleMask.contains(.fullScreen),
              point.y > contentLayoutRect.maxY,
              let hit = contentView?.superview?.hitTest(point)
        else {
            return false
        }
        let appKit = Bundle(for: NSWindow.self)
        return hit === contentView || sequence(first: hit, next: \.superview).allSatisfy { view in
            !(view is NSControl) && !(view is NSText) && Bundle(for: type(of: view)) == appKit
        }
    }

    func performTitleBarDoubleClick() {
        switch UserDefaults.standard.string(forKey: "AppleActionOnDoubleClick") {
            case "Minimize": performMiniaturize(nil)
            case "None": break
            default: performZoom(nil)
        }
    }
}
