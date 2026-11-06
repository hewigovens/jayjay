import AppKit

extension NSEvent {
    var shortcutText: String {
        var parts: [String] = []
        if modifierFlags.contains(.control) { parts.append("ctrl") }
        if modifierFlags.contains(.option) { parts.append("alt") }
        if modifierFlags.contains(.shift) { parts.append("shift") }
        if modifierFlags.contains(.command) { parts.append("cmd") }
        let key: String
        switch keyCode {
            case KeyCode.returnKey, KeyCode.keypadEnter: key = "enter"
            case KeyCode.escape: key = "escape"
            case KeyCode.tab: key = "tab"
            case KeyCode.space: key = "space"
            case KeyCode.delete: key = "backspace"
            case KeyCode.forwardDelete: key = "delete"
            case KeyCode.leftArrow: key = "left"
            case KeyCode.rightArrow: key = "right"
            case KeyCode.downArrow: key = "down"
            case KeyCode.upArrow: key = "up"
            case KeyCode.home: key = "home"
            case KeyCode.end: key = "end"
            case KeyCode.pageUp: key = "pageup"
            case KeyCode.pageDown: key = "pagedown"
            default:
                let characters = charactersIgnoringModifiers?.lowercased() ?? ""
                if let scalar = characters.unicodeScalars.first, (0xF704 ... 0xF70F).contains(scalar.value) {
                    key = "f\(scalar.value - 0xF703)"
                } else {
                    key = characters
                }
        }
        parts.append(key)
        return parts.joined(separator: "-")
    }
}
