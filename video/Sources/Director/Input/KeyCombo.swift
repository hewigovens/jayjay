import CoreGraphics
import Foundation

/// Key codes follow the US layout, whatever layout is active.
struct KeyCombo {
    static let paste = try! KeyCombo("cmd+v")

    let text: String
    let code: CGKeyCode
    let flags: CGEventFlags

    init(_ text: String) throws {
        var parts = text.lowercased().split(separator: "+", omittingEmptySubsequences: false).map(String.init)
        guard let key = parts.popLast(), let code = Self.codes[key] else {
            throw Failure("unknown key in \"\(text)\"; use a letter, digit, punctuation, or a name such as return, escape, tab, space, delete, up, down, left, right, home, end, pageup, pagedown, f1–f12")
        }
        var flags: CGEventFlags = Self.navigation.contains(code) ? [.maskSecondaryFn] : []
        if (123 ... 126).contains(code) {
            flags.insert(.maskNumericPad)
        }
        for part in parts {
            guard let flag = Self.modifiers[part] else {
                throw Failure("unknown modifier \"\(part)\" in \"\(text)\"; use cmd, shift, opt, or ctrl")
            }
            flags.insert(flag)
        }
        self.text = text
        self.code = code
        self.flags = flags
    }

    /// Flags alone latch a modifier for the whole session, so each one is pressed and released.
    func post(check: () throws -> Void) throws {
        let held = Self.modifierKeys.filter { flags.contains($0.flag) }
        var down: CGEventFlags = []
        defer {
            for modifier in held.reversed() where down.contains(modifier.flag) {
                down.remove(modifier.flag)
                Self.post(code: modifier.code, type: .flagsChanged, flags: down)
            }
        }
        for modifier in held {
            down.insert(modifier.flag)
            Self.post(code: modifier.code, type: .flagsChanged, flags: down)
        }
        try check()
        Self.post(code: code, type: .keyDown, flags: flags)
        Self.post(code: code, type: .keyUp, flags: flags)
    }

    static func releaseModifiers() {
        for modifier in modifierKeys {
            post(code: modifier.code, type: .flagsChanged, flags: [])
        }
    }

    private static func post(code: CGKeyCode, type: CGEventType, flags: CGEventFlags) {
        guard let event = CGEvent(keyboardEventSource: nil, virtualKey: code, keyDown: type == .keyDown) else { return }
        event.type = type
        event.flags = flags
        event.post(tap: .cghidEventTap)
        usleep(20000)
    }

    private static let modifierKeys: [(code: CGKeyCode, flag: CGEventFlags)] = [
        (55, .maskCommand), (56, .maskShift), (58, .maskAlternate), (59, .maskControl)
    ]

    private static let modifiers: [String: CGEventFlags] = [
        "cmd": .maskCommand, "command": .maskCommand, "shift": .maskShift,
        "opt": .maskAlternate, "option": .maskAlternate, "alt": .maskAlternate,
        "ctrl": .maskControl, "control": .maskControl
    ]

    /// A keyboard sends these with the fn flag, and arrows also with the keypad flag.
    private static let navigation: Set<CGKeyCode> = [96, 97, 98, 99, 100, 101, 103, 109, 111, 115, 116, 117, 118, 119, 120, 121, 122, 123, 124, 125, 126]

    private static let codes: [String: CGKeyCode] = [
        "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5, "z": 6, "x": 7, "c": 8, "v": 9, "b": 11, "q": 12, "w": 13, "e": 14,
        "r": 15, "y": 16, "t": 17, "1": 18, "2": 19, "3": 20, "4": 21, "6": 22, "5": 23, "=": 24, "9": 25, "7": 26, "-": 27,
        "8": 28, "0": 29, "]": 30, "o": 31, "u": 32, "[": 33, "i": 34, "p": 35, "l": 37, "j": 38, "'": 39, "k": 40, ";": 41,
        "\\": 42, ",": 43, "/": 44, "n": 45, "m": 46, ".": 47, "`": 50,
        "return": 36, "enter": 36, "tab": 48, "space": 49, "delete": 51, "escape": 53, "esc": 53,
        "f1": 122, "f2": 120, "f3": 99, "f4": 118, "f5": 96, "f6": 97, "f7": 98, "f8": 100, "f9": 101, "f10": 109,
        "f11": 103, "f12": 111, "home": 115, "pageup": 116, "forwarddelete": 117, "end": 119, "pagedown": 121,
        "left": 123, "right": 124, "down": 125, "up": 126
    ]
}
