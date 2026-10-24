import JayJayCore
import SwiftUI

struct OverviewLanePanel: View {
    let lane: OverviewLane
    let trunkName: String
    let workspaceInfo: (String) -> WorkspaceInfo?
    let onClose: () -> Void
    let onShowInGraph: () -> Void
    let onRebase: () -> Void
    let onOpenWorkspace: (WorkspaceInfo) -> Void
    let onSelectChange: (OverviewChange) -> Void

    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography

    var body: some View {
        OverviewPanel(accessibilityIdentifier: AID.Overview.lanePanel) {
            OverviewPanelHeader(onClose: onClose) {
                Text(lane.title)
                    .foregroundStyle(lane.head.description.isEmpty ? .tertiary : .primary)
                    .textSelection(.enabled)
            }
            OverviewMetaLine(parts: [
                (Localized.Repo.changeCount(count: lane.changes.count), .secondary),
                (Localized.Overview.updated(age: Date.relativeLabel(millis: lane.latestTimestampMillis)), .secondary),
                (lane.baseSentence(trunkName: trunkName), lane.isBehindTrunk ? .orange : .secondary)
            ])
            HStack(spacing: 6) {
                Button(Localized.Repo.showInGraph, action: onShowInGraph)
                if lane.isBehindTrunk {
                    Button(Localized.Overview.rebaseOnto(trunk: trunkName), action: onRebase)
                }
            }
            .controlSize(.small)
            if !lane.attention.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    ForEach(lane.attention, id: \.self) { sentence in
                        Label(sentence, systemImage: "exclamationmark.triangle.fill")
                            .jayjayFont(11)
                            .foregroundStyle(.orange)
                    }
                }
            }
            if !lane.workspaces.isEmpty {
                OverviewPanelSection(title: Localized.Repo.workspaces) {
                    ForEach(lane.workspaces, id: \.name) { workspace in
                        workspaceRow(workspace)
                    }
                }
            }
            let bookmarks = lane.changes.flatMap(\.bookmarks)
            if !bookmarks.isEmpty {
                OverviewPanelSection(title: Localized.Repo.bookmarks) {
                    FlowLayout(spacing: 4) {
                        ForEach(bookmarks, id: \.self) { bookmark in
                            OverviewChip(text: bookmark, tint: AppColors.bookmark(colorScheme))
                        }
                    }
                }
            }
            OverviewPanelSection(title: Localized.Overview.changesSentence) {
                ForEach(lane.changes, id: \.commitId.id) { change in
                    changeRow(change)
                }
            }
        }
    }

    private func workspaceRow(_ workspace: OverviewWorkspace) -> some View {
        HStack(spacing: 8) {
            OverviewChip(text: workspace.label, tint: AppColors.workspace(colorScheme))
            Text(workspace.changesAbove == 0 ? Localized.Overview.atTheHead : Localized.Overview.belowTheHead(count: Int(workspace.changesAbove)))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer(minLength: 0)
            if !workspace.isCurrent, let info = workspaceInfo(workspace.name), info.isPathResolved {
                Button(Localized.Common.open) { onOpenWorkspace(info) }
                    .controlSize(.small)
            }
        }
    }

    private func changeRow(_ change: OverviewChange) -> some View {
        Button {
            onSelectChange(change)
        } label: {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text(change.changeId.highlighted(
                    scheme: colorScheme,
                    font: typography.font(11, design: .monospaced)
                ))
                Text(change.title)
                    .jayjayFont(12)
                    .foregroundStyle(change.description.isEmpty ? .tertiary : .primary)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(.vertical, 3)
            .padding(.horizontal, 6)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .padding(.horizontal, -6)
    }
}
