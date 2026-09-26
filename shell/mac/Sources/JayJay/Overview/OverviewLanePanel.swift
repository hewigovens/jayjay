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
    @Environment(\.jayjayFontSize) private var baseFontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    var body: some View {
        OverviewPanel(accessibilityIdentifier: AID.Overview.lanePanel) {
            OverviewPanelHeader(onClose: onClose) {
                Text(lane.title)
                    .foregroundStyle(lane.head.description.isEmpty ? .tertiary : .primary)
                    .textSelection(.enabled)
            }
            OverviewMetaLine(parts: [
                ("\(lane.changes.count) change\(lane.changes.count == 1 ? "" : "s")", .secondary),
                ("updated \(Date.relativeLabel(millis: lane.latestTimestampMillis))", .secondary),
                (lane.baseSentence(trunkName: trunkName), lane.isBehindTrunk ? .orange : .secondary)
            ])
            HStack(spacing: 6) {
                Button("Show in Graph", action: onShowInGraph)
                if lane.isBehindTrunk {
                    Button("Rebase onto \(trunkName)", action: onRebase)
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
                OverviewPanelSection(title: "Workspaces") {
                    ForEach(lane.workspaces, id: \.name) { workspace in
                        workspaceRow(workspace)
                    }
                }
            }
            let bookmarks = lane.changes.flatMap(\.bookmarks)
            if !bookmarks.isEmpty {
                OverviewPanelSection(title: "Bookmarks") {
                    FlowLayout(spacing: 4) {
                        ForEach(bookmarks, id: \.self) { bookmark in
                            OverviewChip(text: bookmark, tint: AppColors.bookmark(colorScheme))
                        }
                    }
                }
            }
            OverviewPanelSection(title: "Changes") {
                ForEach(lane.changes, id: \.commitId.id) { change in
                    changeRow(change)
                }
            }
        }
    }

    private func workspaceRow(_ workspace: OverviewWorkspace) -> some View {
        HStack(spacing: 8) {
            OverviewChip(text: workspace.label, tint: AppColors.workspace(colorScheme))
            Text(workspace.changesAbove == 0 ? "at the head" : "\(workspace.changesAbove) below the head")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Spacer(minLength: 0)
            if !workspace.isCurrent, let info = workspaceInfo(workspace.name), info.isPathResolved {
                Button("Open") { onOpenWorkspace(info) }
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
                    font: fontFamily.scaledFont(11, baseSize: baseFontSize, design: .monospaced)
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
