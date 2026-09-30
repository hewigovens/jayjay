import AppKit
import SwiftUI

/// Scoped `NSEvent` keydown monitor — fires only for the containing key window, while the view is enabled, when `isActive`
/// returns true, and the focused text view, if any, does not keep the event.
struct KeyDownMonitor: NSViewRepresentable {
    var isActive: () -> Bool = { true }
    /// Diff views hold selectable read-only NSTextViews; clicking one must not disable list navigation, while editable inputs keep swallowing keys.
    var yieldsToText: (NSText) -> Bool = { _ in true }
    /// Monitors fire in install order, so the window's shortcut monitor must pass j/k on to the pane monitors behind it.
    var swallowsUnhandledKeys = false
    let onKeyDown: (NSEvent) -> Bool

    func makeNSView(context: Context) -> NSView {
        let view = NSView()
        context.coordinator.isEnabled = context.environment.isEnabled
        context.coordinator.install(on: view)
        return view
    }

    func updateNSView(_: NSView, context: Context) {
        context.coordinator.isEnabled = context.environment.isEnabled
        context.coordinator.onKeyDown = onKeyDown
        context.coordinator.isActive = isActive
        context.coordinator.yieldsToText = yieldsToText
        context.coordinator.swallowsUnhandledKeys = swallowsUnhandledKeys
    }

    func makeCoordinator() -> Coordinator {
        let coordinator = Coordinator(isActive: isActive, yieldsToText: yieldsToText, onKeyDown: onKeyDown)
        coordinator.swallowsUnhandledKeys = swallowsUnhandledKeys
        return coordinator
    }

    final class Coordinator {
        var isActive: () -> Bool
        var yieldsToText: (NSText) -> Bool
        var onKeyDown: (NSEvent) -> Bool
        var isEnabled = true
        var swallowsUnhandledKeys = false
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
                      isEnabled,
                      isActive()
                else {
                    return event
                }
                if let text = window.firstResponder as? NSText, yieldsToText(text) {
                    return event
                }
                if onKeyDown(event) {
                    return nil
                }
                if swallowsUnhandledKeys, Self.consumesUnhandledKey(event, firstResponder: window.firstResponder) {
                    return nil
                }
                return event
            }
        }

        /// A key no responder owns reaches `noResponderFor:` and beeps.
        static func consumesUnhandledKey(_ event: NSEvent, firstResponder: NSResponder?) -> Bool {
            guard event.modifierFlags.isDisjoint(with: [.command, .control, .option]),
                  isBeepOnlyKey(event)
            else {
                return false
            }
            if firstResponder is NSText {
                return false
            }
            // A List's NSTableView stays first responder after a click, and the pane's own handlers replace its key handling.
            return firstResponder is NSTableView || !(firstResponder is NSControl)
        }

        private static func isBeepOnlyKey(_ event: NSEvent) -> Bool {
            if [KeyCode.delete, KeyCode.forwardDelete, KeyCode.escape].contains(event.keyCode) {
                return true
            }
            guard let characters = event.characters, !characters.isEmpty else {
                return false
            }
            return characters.unicodeScalars.allSatisfy { scalar in
                !CharacterSet.controlCharacters.contains(scalar) && !(0xF700 ... 0xF8FF).contains(scalar.value)
            }
        }

        deinit {
            if let monitor {
                NSEvent.removeMonitor(monitor)
            }
        }
    }
}
