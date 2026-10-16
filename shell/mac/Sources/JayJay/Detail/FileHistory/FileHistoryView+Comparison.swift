import JayJayCore
import SwiftUI

extension FileHistoryView {
    var comparisonBar: some View {
        HStack(spacing: 6) {
            if let base = comparison.base {
                endpointLabel("Base", base, color: .orange)
                    .accessibilityIdentifier(AID.FileHistory.base)
                Button { comparison.swap() } label: {
                    Image(systemName: "arrow.left.arrow.right")
                }
                .buttonStyle(.borderless)
                .help("Swap base and target")
                .accessibilityIdentifier(AID.FileHistory.swap)
            }
            if let target = comparison.target {
                endpointLabel("Target", target, color: .accentColor)
                    .accessibilityIdentifier(AID.FileHistory.target)
            } else {
                Text("Select a revision")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 8)
            Button("Pin as Base") {
                if let target = comparison.target {
                    comparison.pin(target)
                }
            }
            .disabled(comparison.target == nil)
            .help("Pin the selected revision as the base to compare against")
            if comparison.isPinned {
                Button("Clear") { comparison.clear() }
                    .help("Clear the pinned base (esc)")
            }
        }
    }

    @ViewBuilder
    var comparisonDiff: some View {
        if comparison.isSameRevision {
            ContentUnavailableView(
                "Same Revision as the Base",
                systemImage: "equal.circle",
                description: Text("Choose a different revision to compare against the base.")
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
                "Select a Revision",
                systemImage: "clock",
                description: Text("Pick a revision to see its change to this file.")
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
        .help("\(role): change \(endpoint.changeId.id), commit \(endpoint.commitId.id)")
        .accessibilityElement(children: .combine)
        .accessibilityValue(endpoint.commitId.id)
    }

    @ViewBuilder
    func roleBadge(for change: ChangeInfo) -> some View {
        if comparison.base?.commitId.id == change.commitId.id {
            roleTag("Base", color: .orange)
        } else if comparison.target?.commitId.id == change.commitId.id {
            roleTag("Target", color: .accentColor)
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
