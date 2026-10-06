import AppKit
import SwiftUI

/// A replacement the owner asks the field to make; a new `id` applies it once.
struct FilterFieldEdit: Equatable {
    let id: Int
    let range: NSRange
    let text: String
}

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
    var edit: FilterFieldEdit?
    /// The text and the caret's UTF-16 offset, after typing or moving the caret.
    var onCaretChange: ((String, Int) -> Void)?
    /// Up and Down, for owners that navigate a list of their own; false leaves the key to the field.
    var onMove: ((Int) -> Bool)?
    /// Tab, for owners with a completion to finish; false leaves it to the focus cycle.
    var onComplete: (() -> Bool)?
    /// A click in a window the owner shows for this field, such as a completion list, keeps the edit going.
    var ownsWindow: ((NSWindow?) -> Bool)?
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
        if let edit, context.coordinator.appliedEdit != edit.id {
            context.coordinator.appliedEdit = edit.id
            // Inserting changes the bound text, which a view update must not do.
            DispatchQueue.main.async {
                (field.currentEditor() as? NSTextView)?.insertText(edit.text, replacementRange: edit.range)
            }
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
        var appliedEdit: Int?
        private var clickMonitor: Any?

        init(parent: FilterField) {
            self.parent = parent
        }

        func watchOutsideClicks(of field: NSTextField) {
            clickMonitor = NSEvent.addLocalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self, weak field] event in
                if let field, let window = field.window, field.currentEditor() != nil,
                   self?.parent.ownsWindow?(event.window) != true,
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
            reportCaret(of: field)
        }

        func controlTextDidBeginEditing(_ notification: Notification) {
            inlinePredictions(in: notification, enabled: false)
        }

        func controlTextDidEndEditing(_ notification: Notification) {
            inlinePredictions(in: notification, enabled: true)
            parent.onEndEditing?()
        }

        func control(_ control: NSControl, textView _: NSTextView, doCommandBy selector: Selector) -> Bool {
            switch selector {
                case #selector(NSResponder.cancelOperation(_:)):
                    parent.onCancel()
                case #selector(NSResponder.insertNewline(_:)):
                    parent.onSubmit()
                case #selector(NSResponder.moveUp(_:)), #selector(NSResponder.moveDown(_:)):
                    return parent.onMove?(selector == #selector(NSResponder.moveUp(_:)) ? -1 : 1) ?? false
                case #selector(NSResponder.insertTab(_:)):
                    return parent.onComplete?() ?? false
                default:
                    // The caret has not moved yet when the command arrives.
                    DispatchQueue.main.async { [weak self, weak control] in
                        if let field = control as? NSTextField {
                            self?.reportCaret(of: field)
                        }
                    }
                    return false
            }
            return true
        }

        private func reportCaret(of field: NSTextField) {
            guard let onCaretChange = parent.onCaretChange, let editor = field.currentEditor() else { return }
            onCaretChange(field.stringValue, editor.selectedRange.location)
        }

        /// The field editor is shared with the window's other fields, so predictions end with the session.
        private func inlinePredictions(in notification: Notification, enabled: Bool) {
            guard let editor = (notification.object as? NSTextField)?.currentEditor() as? NSTextView else {
                return
            }
            editor.inlinePredictionType = enabled ? .default : .no
        }
    }
}
