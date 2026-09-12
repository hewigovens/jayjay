import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func descriptionSection() -> some View {
        DetailDescriptionSection(
            description: detail.info.description,
            expanded: $descriptionExpanded,
            isImmutable: detail.info.isImmutable,
            canEditDescription: !detail.info.isWorkingCopy && !detail.info.isImmutable,
            canShowDiffEditButton: canShowDiffEditButton,
            onEdit: { onEditDescription(detailRevision, detail.info.description) },
            onOpenDiffEdit: { paneMode = .diffEdit }
        )
        .id("\(detailRevision)|\(detail.info.commitId)")
    }

    static func canEnterDiffEdit(info: ChangeInfo, isCompareMode: Bool) -> Bool {
        !isCompareMode && !info.hasConflict && !info.isImmutable && !info.isEmpty
    }

    var canEnterDiffEdit: Bool {
        Self.canEnterDiffEdit(info: detail.info, isCompareMode: isCompareMode)
    }

    private var canShowDiffEditButton: Bool {
        canEnterDiffEdit && !detail.diff.isEmpty
    }
}

private struct DetailDescriptionSection: View {
    private enum Metrics {
        static let maximumHeight: CGFloat = 80
    }

    let description: String
    @Binding var expanded: Bool
    @State private var overflows = false
    let isImmutable: Bool
    let canEditDescription: Bool
    let canShowDiffEditButton: Bool
    let onEdit: () -> Void
    let onOpenDiffEdit: () -> Void

    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            if description.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
                if canEditDescription {
                    Button("Add description", systemImage: "pencil", action: onEdit)
                        .buttonStyle(.plain)
                        .keyboardFocusStop(.editDescription, action: onEdit)
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                } else if isImmutable {
                    Text("No description")
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                }
                Spacer(minLength: 0)
            } else {
                DescriptionPreview(
                    description: description,
                    maximumHeight: Metrics.maximumHeight,
                    expanded: expanded,
                    onEdit: canEditDescription ? onEdit : nil,
                    onOverflowChanged: { overflows = $0 }
                )
            }
            Button {
                expanded.toggle()
            } label: {
                Image(systemName: expanded ? "arrow.down.and.line.horizontal.and.arrow.up" : "arrow.up.and.line.horizontal.and.arrow.down")
            }
            .buttonStyle(.plain)
            .frame(width: 18, height: 18)
            .foregroundStyle(.secondary)
            .keyboardFocusStop(.expandDescription, isAvailable: overflows || expanded) { expanded.toggle() }
            .accessibilityIdentifier(AID.Detail.descriptionExpansion)
            .accessibilityLabel(expanded ? "Collapse description" : "Expand description")
            .help(expanded ? "Collapse description" : "Expand description")
            .opacity(overflows || expanded ? 1 : 0)
            .disabled(!overflows && !expanded)
            .accessibilityHidden(!overflows && !expanded)
            if canShowDiffEditButton {
                Button("Edit Diff...", action: onOpenDiffEdit)
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .keyboardFocusStop(.editDiff, action: onOpenDiffEdit)
                    .accessibilityIdentifier(AID.DiffEdit.open)
                    .help("Open dedicated diff edit mode")
            }
        }
    }
}
