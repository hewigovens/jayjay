import AppKit
import JayJayCore

extension SelectionClick {
    init(modifiers: NSEvent.ModifierFlags) {
        let modifiers = modifiers.intersection(.deviceIndependentFlagsMask)
        if modifiers.contains(.command) {
            self = .toggle
        } else if modifiers.contains(.shift) {
            self = .extend
        } else {
            self = .replace
        }
    }
}
