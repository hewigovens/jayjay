import JayJayCore
import SwiftUI

struct OverviewLaneCard: View {
    let lane: OverviewLane
    let trunkName: String
    let isSelected: Bool

    @Environment(\.colorScheme) private var colorScheme

    private var needsAttention: Bool {
        !lane.attention.isEmpty
    }

    /// Selection wins over attention; the orange sentence on the card still says why.
    private var borderColor: Color {
        if isSelected {
            return .accentColor
        }
        if needsAttention {
            return .orange
        }
        return .primary.opacity(0.12)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(lane.title)
                .jayjayFont(12, weight: .semibold)
                .foregroundStyle(lane.head.description.isEmpty ? .tertiary : .primary)
                .lineLimit(2)
                .frame(maxWidth: .infinity, alignment: .leading)
                .frame(height: 30, alignment: .top)
            HStack(spacing: 4) {
                if lane.workspaces.isEmpty {
                    Text("no workspace")
                        .jayjayFont(10)
                        .foregroundStyle(.tertiary)
                } else {
                    ForEach(lane.workspaces, id: \.name) { workspace in
                        OverviewChip(text: workspace.isCurrent ? "@ \(workspace.name)" : "\(workspace.name)@", tint: AppColors.workspace(colorScheme))
                            .fixedSize()
                    }
                }
                ForEach(lane.changes.flatMap(\.bookmarks), id: \.self) { bookmark in
                    OverviewChip(text: bookmark, tint: AppColors.bookmark(colorScheme))
                }
                Spacer(minLength: 4)
                Text("\(lane.changes.count) change\(lane.changes.count == 1 ? "" : "s") · \(Date.relativeLabel(millis: lane.latestTimestampMillis))")
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .frame(height: 16)
            .clipped()
            Text(lane.attention.first ?? lane.baseSentence(trunkName: trunkName))
                .jayjayFont(10)
                .foregroundStyle(needsAttention || lane.isBehindTrunk ? Color.orange : Color.secondary)
                .lineLimit(1)
        }
        .padding(.leading, 12)
        .padding(.trailing, 10)
        .padding(.vertical, 8)
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 8)
                .fill(Color(nsColor: .controlBackgroundColor))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(borderColor, lineWidth: isSelected || needsAttention ? 2 : 1)
        )
        .contentShape(Rectangle())
    }
}

struct OverviewChip: View {
    let text: String
    let tint: Color

    var body: some View {
        Text(text)
            .jayjayFont(9, weight: .semibold, design: .monospaced)
            .lineLimit(1)
            .truncationMode(.middle)
            .padding(.horizontal, 5)
            .padding(.vertical, 1)
            .background(tint.opacity(0.16), in: RoundedRectangle(cornerRadius: 4))
            .foregroundStyle(tint)
    }
}
