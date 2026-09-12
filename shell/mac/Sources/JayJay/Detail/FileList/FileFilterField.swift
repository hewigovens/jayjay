import AppKit
import SwiftUI

/// AppKit search field: a SwiftUI TextField in the repo window ignores programmatic focus and never forwards Escape to `onExitCommand`; the field exists only while the filter is open, so it takes focus on creation.
struct FileFilterField: NSViewRepresentable {
    @Binding var text: String
    let onSubmit: () -> Void
    let onCancel: () -> Void

    func makeNSView(context: Context) -> NSSearchField {
        let field = NSSearchField()
        field.placeholderString = "Filter files"
        field.controlSize = .small
        field.font = .systemFont(ofSize: 11)
        field.delegate = context.coordinator
        field.setAccessibilityIdentifier(AID.FileList.filterField)
        field.setContentHuggingPriority(.defaultLow, for: .horizontal)
        field.setContentCompressionResistancePriority(.defaultLow, for: .horizontal)
        (field.cell as? NSSearchFieldCell)?.cancelButtonCell = nil
        DispatchQueue.main.async { field.window?.makeFirstResponder(field) }
        return field
    }

    func updateNSView(_ field: NSSearchField, context: Context) {
        context.coordinator.parent = self
        if field.stringValue != text {
            field.stringValue = text
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator(parent: self)
    }

    final class Coordinator: NSObject, NSSearchFieldDelegate {
        var parent: FileFilterField

        init(parent: FileFilterField) {
            self.parent = parent
        }

        func controlTextDidChange(_ notification: Notification) {
            guard let field = notification.object as? NSSearchField else { return }
            parent.text = field.stringValue
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
