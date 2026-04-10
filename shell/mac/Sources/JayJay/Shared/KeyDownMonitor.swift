import AppKit
import SwiftUI

/// Scoped `NSEvent` keydown monitor — fires only for the containing key window, when `isActive` returns true, and no
/// text input owns focus.
struct KeyDownMonitor: NSViewRepresentable {
    var isActive: () -> Bool = { true }
    let onKeyDown: (NSEvent) -> Bool

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.install(on: view)
        return view
    }

    func updateNSView(_: NSView, context: Context) {
        context.coordinator.onKeyDown = onKeyDown
        context.coordinator.isActive = isActive
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(isActive: isActive, onKeyDown: onKeyDown)
    }

    final class Coordinator {
        var isActive: () -> Bool
        var onKeyDown: (NSEvent) -> Bool
        private weak var view: NSView?
        private var monitor: Any?

        init(isActive: @escaping () -> Bool, onKeyDown: @escaping (NSEvent) -> Bool) {
            self.isActive = isActive
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
                if let responder = window.firstResponder,
                   responder is NSText || responder is NSTextView
                {
                    return event
                }
                return onKeyDown(event) ? nil : event
            }
        }

        deinit {
            if let monitor { NSEvent.removeMonitor(monitor) }
        }
    }
}
