import AppKit
import SwiftUI

/// SwiftUI's TextField ignores programmatic focus and Escape in the repo window.
struct FilterField: NSViewRepresentable {
    @Binding var text: String
    var placeholder = "Filter files"
    var accessibilityIdentifier = AID.FileList.filterField
    var focusGeneration = 0
    var isPlain = false
    var endsEditingOnOutsideClick = false
    var placesCaretAtEnd = false
    var font: NSFont?
    let onSubmit: () -> Void
    let onCancel: () -> Void
    var onEndEditing: (() -> Void)?

    func makeNSView(context: Context) -> NSTextField {
        let field = isPlain ? Self.plainField() : Self.searchField()
        field.placeholderString = placeholder
        if let font {
            field.font = font
        }
        field.delegate = context.coordinator
        field.setAccessibilityIdentifier(accessibilityIdentifier)
        field.setContentHuggingPriority(.defaultLow, for: .horizontal)
        field.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        DispatchQueue.main.async { [placesCaretAtEnd] in
            field.window?.makeFirstResponder(field)
            if placesCaretAtEnd, let editor = field.currentEditor() {
                editor.selectedRange = NSRange(location: field.stringValue.utf16.count, length: 0)
                editor.scrollRangeToVisible(editor.selectedRange)
            }
        }
        if endsEditingOnOutsideClick {
            context.coordinator.watchOutsideClicks(of: field)
        }
        return field
    }

    static func dismantleNSView(_: NSTextField, coordinator: Coordinator) {
        coordinator.stopWatchingClicks()
    }

    func updateNSView(_ field: NSTextField, context: Context) {
        let refocus = context.coordinator.parent.focusGeneration != focusGeneration
        context.coordinator.parent = self
        if field.stringValue != text {
            field.stringValue = text
        }
        if let font, field.font != font {
            field.font = font
        }
        if refocus {
            field.window?.makeFirstResponder(field)
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    private static func searchField() -> NSTextField {
        let field = NSSearchField()
        field.controlSize = .small
        field.font = .systemFont(ofSize: 11)
        (field.cell as? NSSearchFieldCell)?.cancelButtonCell = nil
        return field
    }

    private static func plainField() -> NSTextField {
        let field = NSTextField()
        field.isBezeled = false
        field.isBordered = false
        field.drawsBackground = false
        field.focusRingType = .none
        field.usesSingleLineMode = true
        field.cell?.isScrollable = true
        return field
    }

    final class Coordinator: NSObject, NSTextFieldDelegate {
        var parent: FilterField
        private var clickMonitor: Any?

        init(parent: FilterField) {
            self.parent = parent
        }

        func watchOutsideClicks(of field: NSTextField) {
            clickMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak field] event in
                if let field, let window = field.window, field.currentEditor() != nil,
                   event.window !== window || !field.bounds.contains(field.convert(event.locationInWindow, from: nil))
                {
                    window.makeFirstResponder(nil)
                }
                return event
            }
        }

        func stopWatchingClicks() {
            if let clickMonitor {
                NSEvent.removeMonitor(clickMonitor)
                self.clickMonitor = nil
            }
        }

        func controlTextDidChange(_ notification: Notification) {
            guard let field = notification.object as? NSTextField else { return }
            parent.text = field.stringValue
        }

        func controlTextDidEndEditing(_: Notification) {
            parent.onEndEditing?()
        }

        func control(_: NSControl, textView _: NSTextView, doCommandBy selector: Selector) -> Bool {
            switch selector {
                case #selector(NSResponder.cancelOperation(_:)):
                    parent.onCancel()
                case #selector(NSResponder.insertNewline(_:)):
                    parent.onSubmit()
                default:
                    return false
            }
            return true
        }
    }
}
