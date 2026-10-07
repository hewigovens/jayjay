import AppKit

extension NSEvent {
    var horizontalNavigationKeyCode: UInt16 {
        guard modifierFlags.isDisjoint(with: [.command, .control, .option, .shift]) else {
            return keyCode
        }
        switch charactersIgnoringModifiers {
            case "h": return KeyCode.leftArrow
            case "l": return KeyCode.rightArrow
            default: return keyCode
        }
    }
}
