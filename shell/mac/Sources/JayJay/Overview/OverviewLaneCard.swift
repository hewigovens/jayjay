import JayJayCore
import SwiftUI

struct OverviewLaneCard: View {
    let lane: OverviewLane
    let isSelected: Bool

    @Environment(\.colorScheme) private var colorScheme

    private var needsAttention: Bool {
        !lane.attention.isEmpty
    }

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
            OverviewChipRow(chips: chips)
                .frame(height: 16)
            Text(lane.footer)
                .jayjayFont(10)
                .foregroundStyle(needsAttention ? Color.orange : Color.secondary)
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

private extension OverviewLaneCard {
    var chips: [OverviewChipRow.Chip] {
        let workspace = AppColors.workspace(colorScheme)
        let bookmark = AppColors.bookmark(colorScheme)
        return lane.workspaces.map { .init(text: $0.label, tint: workspace) }
            + lane.changes.flatMap(\.bookmarks).map { .init(text: $0, tint: bookmark) }
    }
}
