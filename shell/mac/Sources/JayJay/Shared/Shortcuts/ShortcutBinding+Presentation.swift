import JayJayCore

extension ShortcutBinding: @retroactive Identifiable {}

extension ShortcutBinding {
    var effectiveChords: [ShortcutChord] {
        [primary].compactMap { $0 } + aliases
    }

    var chords: [[String]] {
        effectiveChords.map(\.caps)
    }
}
