import Foundation
import JayJayCore
import Observation

@Observable
final class ShortcutStore {
    static let shared = ShortcutStore()
    static let storageKey = "jayjay.shortcutOverrides"

    private(set) var stored: [String: [String]]
    private(set) var bindings: [ShortcutBinding]
    private(set) var byID: [String: ShortcutBinding]
    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        let stored = defaults.dictionary(forKey: Self.storageKey) as? [String: [String]] ?? [:]
        self.stored = stored
        let bindings = shortcutBindings(stored: stored)
        self.bindings = bindings
        byID = Dictionary(uniqueKeysWithValues: bindings.map { ($0.id, $0) })
    }

    func apply(_ edit: ShortcutEdit, to id: String = "", replace: Bool = false) throws {
        let updated = try editShortcut(stored: stored, id: id, edit: edit, replace: replace)
        defaults.set(updated, forKey: Self.storageKey)
        stored = updated
        bindings = shortcutBindings(stored: updated)
        byID = Dictionary(uniqueKeysWithValues: bindings.map { ($0.id, $0) })
    }
}
