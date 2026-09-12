import AppKit
import SwiftUI

/// Scoped `NSEvent` keydown monitor — fires only for the containing key window, when `isActive` returns true, and the
/// focused text view, if any, does not keep the event.
struct KeyDownMonitor: NSViewRepresentable {
    var isActive: () -> Bool = { true }
    /// Diff views hold selectable read-only NSTextViews; clicking one must not disable list navigation, while editable inputs keep swallowing keys.
    var yieldsToText: (NSText) -> Bool = { _ in true }
    let onKeyDown: (NSEvent) -> Bool

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }

    func updateNSView(_: NSView, context: Context) {
        context.coordinator.onKeyDown = onKeyDown
        context.coordinator.isActive = isActive
        context.coordinator.yieldsToText = yieldsToText
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(isActive: isActive, yieldsToText: yieldsToText, onKeyDown: onKeyDown)
    }

    final class Coordinator {
        var isActive: () -> Bool
        var yieldsToText: (NSText) -> Bool
        var onKeyDown: (NSEvent) -> Bool
        private weak var view: NSView?
        private var monitor: Any?

        init(
            isActive: @escaping () -> Bool,
            yieldsToText: @escaping (NSText) -> Bool,
            onKeyDown: @escaping (NSEvent) -> Bool
        ) {
            self.isActive = isActive
            self.yieldsToText = yieldsToText
            self.onKeyDown = onKeyDown
        }

        func install(on view: NSView) {
            self.view = view
            monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
                guard let self,
                      let view = self.view,
                      let window = view.window,
                      NSApp.keyWindow === window,
                      isActive()
                else {
                    return event
                }
                if let text = window.firstResponder as? NSText, yieldsToText(text) {
                    return event
                }
                return onKeyDown(event) ? nil : event
            }
        }

        deinit {
            if let monitor {
                NSEvent.removeMonitor(monitor)
            }
        }
    }
}
