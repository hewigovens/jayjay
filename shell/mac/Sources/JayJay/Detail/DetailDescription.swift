import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func descriptionSection() -> some View {
        DetailDescriptionSection(
            description: detail.info.description,
            expanded: descriptionExpanded,
            expandedHeight: DescriptionHeight.expanded(paneHeight: paneHeight),
            canEditDescription: !detail.info.isWorkingCopy && !detail.info.isImmutable,
            onEdit: { onEditDescription(detailRevision, detail.info.description) }
        )
        .id("\(detailRevision)|\(detail.info.commitId)")
    }

    static func canEnterDiffEdit(info: ChangeInfo, isCompareMode: Bool) -> Bool {
        !isCompareMode && !info.hasConflict && !info.isImmutable && !info.isEmpty
    }

    var canEnterDiffEdit: Bool {
        Self.canEnterDiffEdit(info: detail.info, isCompareMode: isCompareMode)
    }
}

private struct DetailDescriptionSection: View {
    let description: String
    @Binding var expanded: Bool
    let expandedHeight: CGFloat
    let canEditDescription: Bool
    let onEdit: () -> Void

    var body: some View {
        if description.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            HStack(alignment: .firstTextBaseline, spacing: 8) {
                Text("No description")
                    .jayjayFont(.title)
                    .foregroundStyle(.secondary)
                if canEditDescription {
                    Button("Add description", systemImage: "pencil", action: onEdit)
                        .buttonStyle(.plain)
                        .keyboardFocusStop(.editDescription, action: onEdit)
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                }
                DescriptionExpansionToggle(expanded: expanded) { expanded.toggle() }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        } else {
            DescriptionPreview(
                description: description,
                collapsedHeight: DescriptionHeight.collapsed,
                expandedHeight: expandedHeight,
                expanded: expanded,
                onEdit: canEditDescription ? onEdit : nil,
                onToggleExpansion: { expanded.toggle() }
            )
        }
    }
}
