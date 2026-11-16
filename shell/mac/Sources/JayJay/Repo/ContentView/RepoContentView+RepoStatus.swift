import JayJayCore
import SwiftUI

/// Repo-level status that used to sit in a status bar, each piece now shown at the level it belongs to.
extension RepoContentView {
    /// SSH host and connection state, shown in the title subtitle for remote repositories.
    var sshContext: String? {
        sshMachine(location: viewModel.repoPath).map { "\($0) · \(viewModel.sshStatusLabel)" }
    }

    /// Conflict and divergence counts as filter buttons in the change list header; the plain change count when there are none.
    @ViewBuilder var changeListStatus: some View {
        let conflicted = viewModel.changes.filter(\.hasConflict).count
        let divergent = Set(viewModel.changes.filter(\.isDivergent).map(\.changeId.id)).count
        if conflicted == 0, divergent == 0 {
            Text(Localized.Repo.changeCount(count: viewModel.changes.count))
                .foregroundStyle(.secondary)
        } else {
            HStack(spacing: 8) {
                if conflicted > 0, let conflicts = RevsetFilterPresets.preset(id: "conflicts") {
                    Button(Localized.Repo.conflictedCount(count: conflicted), systemImage: "exclamationmark.triangle.fill") {
                        viewModel.applyFilter(conflicts.revset)
                    }
                    .foregroundStyle(.red)
                }
                if divergent > 0 {
                    Button(Localized.Repo.divergentCount(count: divergent), systemImage: "arrow.triangle.branch") {
                        viewModel.applyFilter("divergent()")
                    }
                    .foregroundStyle(.orange)
                }
            }
            .buttonStyle(.plain)
        }
    }
}

extension PrInfo {
    var stateLabel: String {
        switch state {
            case .open: Localized.Repo.prOpen
            case .closed: Localized.Repo.closed
            case .merged: Localized.Repo.merged
        }
    }

    var checksIcon: String {
        switch checks {
            case .passing: "checkmark.circle.fill"
            case .failing: "xmark.circle.fill"
            case .pending: "clock.circle"
            case .none: "circle.dashed"
        }
    }
}
