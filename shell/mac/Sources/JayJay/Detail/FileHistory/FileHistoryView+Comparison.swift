import JayJayCore
import SwiftUI

extension FileHistoryView {
    var comparisonBar: some View {
        HStack(spacing: 6) {
            if let base = comparison.base {
                endpointLabel(Localized.Repo.base, base, color: .orange)
                    .accessibilityIdentifier(AID.FileHistory.base)
                Button { comparison.swap() } label: {
                    Image(systemName: "arrow.left.arrow.right")
                }
                .buttonStyle(.borderless)
                .help(Localized.Repo.swapBaseAndTarget)
                .accessibilityIdentifier(AID.FileHistory.swap)
            }
            if let target = comparison.target {
                endpointLabel(Localized.Repo.target, target, color: .accentColor)
                    .accessibilityIdentifier(AID.FileHistory.target)
            } else {
                Text(Localized.Repo.selectARevision)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 8)
            Button(Localized.Repo.pinAsBase) {
                if let target = comparison.target {
                    comparison.pin(target)
                }
            }
            .accessibilityIdentifier(AID.FileHistory.pinBase)
            .disabled(comparison.target == nil)
            .help(Localized.Repo.pinTheSelectedRevisionAsThe)
            if comparison.isPinned {
                Button(Localized.Common.clear) { comparison.clear() }
                    .help(Localized.Repo.clearThePinnedBaseEsc)
            }
        }
    }

    @ViewBuilder
    var comparisonDiff: some View {
        if comparison.isSameRevision {
            ContentUnavailableView(
                Localized.Repo.sameRevisionAsTheBase,
                systemImage: "equal.circle",
                description: Text(Localized.Repo.chooseADifferentRevisionToCompare)
            )
        } else if let target = comparison.compareTo {
            FileHistoryComparisonDiff(
                from: comparison.compareFrom,
                to: target,
                path: path,
                repo: repo,
                diffStore: diffStore
            )
            // A fresh identity per endpoint pair drops the previous pair's rendered diff instead of leaving it under the new labels while it reloads.
            .id("\(comparison.compareFrom ?? "")|\(target)|\(path)")
        } else {
            ContentUnavailableView(
                Localized.Repo.selectARevisionTitle,
                systemImage: "clock",
                description: Text(Localized.Repo.pickARevisionToSeeIts)
            )
        }
    }

    func endpoint(for change: ChangeInfo) -> FileHistoryEndpoint {
        FileHistoryEndpoint(commitId: change.commitId, changeId: change.changeId)
    }

    func pin(_ change: ChangeInfo) {
        comparison.pin(endpoint(for: change))
    }

    func reveal(_ change: ChangeInfo) {
        onSelectChange(change.selectionRevision)
    }

    func endpointLabel(_ role: String, _ endpoint: FileHistoryEndpoint, color: Color) -> some View {
        HStack(spacing: 4) {
            Text(role)
                .jayjayFont(10, weight: .semibold)
                .foregroundStyle(color)
            Text(endpoint.changeId.highlighted(scheme: colorScheme, font: typography.identifierFont))
            Text(endpoint.commitId.compact)
                .jayjayFont(11, design: .monospaced)
                .foregroundStyle(.secondary)
        }
        .help(Localized.Repo.comparisonEndpointHelp(role: role, change: endpoint.changeId.id, commit: endpoint.commitId.id))
        .accessibilityElement(children: .combine)
        .accessibilityValue(endpoint.commitId.id)
    }

    @ViewBuilder
    func roleBadge(for change: ChangeInfo) -> some View {
        if comparison.base?.commitId.id == change.commitId.id {
            roleTag(Localized.Repo.base, color: .orange)
        } else if comparison.target?.commitId.id == change.commitId.id {
            roleTag(Localized.Repo.target, color: .accentColor)
        }
    }

    private func roleTag(_ label: String, color: Color) -> some View {
        Text(label)
            .jayjayFont(9, weight: .semibold)
            .foregroundStyle(color)
            .padding(.horizontal, 5)
            .padding(.vertical, 1)
            .background(color.opacity(0.15), in: Capsule())
    }
}
