import AppKit
import JayJayCore
import SwiftUI

extension ShortcutChord {
    var keyboardShortcut: KeyboardShortcut? {
        var parts = text.split(separator: "-", omittingEmptySubsequences: false).map(String.init)
        let modifierTokens: [String: EventModifiers] = ["ctrl": .control, "alt": .option, "shift": .shift, "cmd": .command]
        var modifiers: EventModifiers = []
        while let part = parts.first, let modifier = modifierTokens[part] {
            modifiers.insert(modifier)
            parts.removeFirst()
        }
        let token = parts.joined(separator: "-")
        let key: KeyEquivalent
        switch token {
            case "enter": key = .return
            case "escape": key = .escape
            case "tab": key = .tab
            case "space": key = .space
            case "backspace": key = .delete
            case "delete": key = .deleteForward
            case "left": key = .leftArrow
            case "right": key = .rightArrow
            case "up": key = .upArrow
            case "down": key = .downArrow
            case "home": key = .home
            case "end": key = .end
            case "pageup": key = .pageUp
            case "pagedown": key = .pageDown
            default:
                if token.hasPrefix("f"), let number = UInt32(token.dropFirst()), (1 ... 12).contains(number),
                   let scalar = UnicodeScalar(0xF703 + number) {
                    key = KeyEquivalent(Character(scalar))
                } else if token.count == 1, let character = token.first {
                    key = KeyEquivalent(character)
                } else {
                    return nil
                }
        }
        return KeyboardShortcut(key, modifiers: modifiers)
    }
}
