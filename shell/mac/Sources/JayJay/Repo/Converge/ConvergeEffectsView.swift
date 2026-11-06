import JayJayCore
import SwiftUI

struct ConvergeEffectsView: View {
    let effects: ConvergeEffects

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            if let reason = effects.blockedReason {
                Label(Localized.Converge.blocked, systemImage: "exclamationmark.triangle")
                    .jayjayFont(12, weight: .semibold)
                    .foregroundStyle(.red)
                Text(Localized.Converge.immutableReason)
                DisclosureGroup(Localized.Converge.details) {
                    Text(reason).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            if effects.descendants.isEmpty, effects.bookmarks.isEmpty, effects.workspaces.isEmpty {
                Text(Localized.Converge.noEffects).foregroundStyle(.secondary)
            }
            if !effects.descendants.isEmpty {
                DisclosureGroup(Localized.Converge.rebaseDescendants(count: effects.descendants.count)) {
                    VStack(alignment: .leading, spacing: 4) {
                        ForEach(effects.descendants, id: \.commitId) { descendant in
                            Text("\(Text(String(descendant.commitId.prefix(12))).monospaced())  \(descendant.description.isEmpty ? Localized.Converge.noDescription : descendant.description)")
                        }
                    }
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            names(Localized.Converge.moveBookmarks(count: effects.bookmarks.count), values: effects.bookmarks)
            names(Localized.Converge.moveWorkspaces(count: effects.workspaces.count), values: effects.workspaces)
            Text(Localized.Converge.undoHint).foregroundStyle(.tertiary)
        }
        .jayjayFont(11)
    }

    @ViewBuilder
    private func names(_ title: String, values: [String]) -> some View {
        if !values.isEmpty {
            DisclosureGroup(title) {
                Text(values.joined(separator: "\n"))
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
        }
    }
}
