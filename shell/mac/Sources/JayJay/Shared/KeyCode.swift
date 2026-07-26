import Foundation

/// Hardware key codes (Carbon's kVK_* values), which NSEvent still reports.
enum KeyCode {
    static let returnKey: UInt16 = 36
    static let space: UInt16 = 49
    static let keypadEnter: UInt16 = 76
    static let leftArrow: UInt16 = 123
    static let rightArrow: UInt16 = 124
    static let downArrow: UInt16 = 125
    static let upArrow: UInt16 = 126
}
