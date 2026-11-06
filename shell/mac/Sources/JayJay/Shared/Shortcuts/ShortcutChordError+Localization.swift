import JayJayCore

extension ShortcutChordError {
    var localizedDescription: String {
        switch self {
            case .malformed: Localized.Shortcuts.invalidSyntax
            case .needsModifier: Localized.Shortcuts.needsModifier
            case .reserved: Localized.Shortcuts.reserved
            case .unsupportedModifier: Localized.Shortcuts.unsupportedModifier
        }
    }
}
