import JayJayCore
import SwiftUI

@MainActor
@Observable
final class FlockModel {
    private(set) var member: FlockMember?
    private(set) var icon: FlockIcon
    private let defaults: UserDefaults

    init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
        icon = flockIcon(id: defaults.string(forKey: "jayjay.flockIcon") ?? "")
    }

    var isUnlocked: Bool {
        member != nil
    }

    var activeIcon: FlockIcon {
        isUnlocked ? icon : .classic
    }

    var name: String {
        let name = member?.name?.trimmingCharacters(in: .whitespacesAndNewlines) ?? ""
        return name.isEmpty ? (member?.login ?? "") : name
    }

    func load() async {
        member = await Task.detached(priority: .utility) { flockMember() }.value
        applyIcon()
    }

    func accept(_ member: FlockMember) {
        self.member = member
        applyIcon()
    }

    func forget() async throws {
        try await Task.detached(priority: .userInitiated) { try forgetFlock() }.value
        member = nil
        applyIcon()
    }

    func choose(_ icon: FlockIcon) {
        guard isUnlocked else { return }
        self.icon = icon
        defaults.set(icon.id, forKey: "jayjay.flockIcon")
        applyIcon()
    }

    private func applyIcon() {
        let variant = activeIcon == .classic ? nil : activeIcon.id
        defaults.set(variant, forKey: DockIconView.preferenceKey)
        DockIcon.update(variant: variant)
    }
}
