import AppKit

extension NSTextStorage {
    func replaceContents(with text: NSAttributedString) {
        // Clear existing attribute runs first to avoid costly run-array shifts during full replacement.
        beginEditing()
        deleteCharacters(in: NSRange(location: 0, length: length))
        append(text)
        endEditing()
    }
}
