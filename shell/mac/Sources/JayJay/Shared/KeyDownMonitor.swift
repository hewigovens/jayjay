import AppKit
import SwiftUI

/// Scoped `NSEvent` keydown monitor — fires only for the containing key window, when `isActive` returns true, and no
/// text input owns focus.
struct KeyDownMonitor: NSViewRepresentable {
    var isActive: () -> Bool = { true }
    /// Diff views hold selectable read-only NSTextViews; clicking one must not disable list navigation, while editable inputs keep swallowing keys.
    var ignoresReadOnlyText = false
    let onKeyDown: (NSEvent) -> Bool

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }

    func updateNSView(_: NSView, context: Context) {
        context.coordinator.onKeyDown = onKeyDown
        context.coordinator.isActive = isActive
        context.coordinator.ignoresReadOnlyText = ignoresReadOnlyText
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(
            isActive: isActive, ignoresReadOnlyText: ignoresReadOnlyText, onKeyDown: onKeyDown
        )
    }

    final class Coordinator {
        var isActive: () -> Bool
        var ignoresReadOnlyText: Bool
        var onKeyDown: (NSEvent) -> Bool
        private weak var view: NSView?
        private var monitor: Any?

        init(
            isActive: @escaping () -> Bool,
            ignoresReadOnlyText: Bool,
            onKeyDown: @escaping (NSEvent) -> Bool
        ) {
            self.isActive = isActive
            self.ignoresReadOnlyText = ignoresReadOnlyText
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
                if let text = window.firstResponder as? NSText,
                   text.isEditable || !ignoresReadOnlyText
                {
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
