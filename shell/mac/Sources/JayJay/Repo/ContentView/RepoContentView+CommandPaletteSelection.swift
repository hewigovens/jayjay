import JayJayCore
import SwiftUI

extension RepoContentView {
    func changePaletteItems(change: ChangeInfo) -> [CommandPaletteItem] {
        let target = String(change.changeId.id.prefix(8))
        return availableChangeCommands(change: change)
            .filter { $0 != .fix || viewModel.fixUnavailableReason == nil }
            .map { command in
                CommandPaletteItem(
                    title: command.title,
                    icon: command.systemImage,
                    category: Localized.Repo.change,
                    detail: target,
                    keywords: command.paletteKeywords,
                    shortcut: command.shortcut?.symbol
                ) { command.perform(on: change, actions: viewModel, onRequest: handleDAGRequest) }
            }
    }

    var multiSelectionPaletteItems: [CommandPaletteItem] {
        let revisions = viewModel.selectedChangeIds
        guard revisions.count > 1 else {
            return []
        }
        let capabilities = viewModel.selectionCapabilities
        return selectionCommands()
            .filter { capabilities.isAvailable($0) && ($0 != .fix || viewModel.fixUnavailableReason == nil) }
            .flatMap { command in
                let choices: [(parents: [String], label: String?)] = command == .merge
                    ? mergeParentChoices(changes: viewModel.changes.filter { revisions.contains($0.selectionRevision) })
                    .map { ($0.parents, $0.label) }
                    : [(revisions, nil)]
                return choices.map { parents, detail in
                    CommandPaletteItem(
                        title: command.title(count: revisions.count),
                        icon: command.systemImage,
                        category: Localized.Repo.change,
                        detail: detail,
                        keywords: ["selection", "stack"]
                    ) { command.perform(on: parents, actions: viewModel, onRequest: handleDAGRequest) }
                }
            }
    }
}

private extension ChangeCommand {
    var paletteKeywords: [String] {
        switch self {
            case .fix: ["format", "formatter", "fix", "lint"]
            case .showEvolution: ["evolog", "obslog", "history"]
            case .moveChangesToWorkingCopy: ["squash", "into"]
            case .rebaseOntoTrunk: ["main", "update"]
            default: []
        }
    }
}
