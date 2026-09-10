import JayJayCore
import SwiftUI

extension ChangeDetailView {
    func descriptionSection() -> some View {
        DetailDescriptionSection(
            description: detail.info.description,
            descriptionText: $descriptionText,
            editingDescription: $editingDescription,
            canEditDescription: !detail.info.isWorkingCopy,
            canShowDiffEditButton: canShowDiffEditButton,
            onSave: { onDescribe(detailRevision, $0) },
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
        canEnterDiffEdit && !detail.diff.isEmpty && !editingDescription
    }
}

private struct DetailDescriptionSection: View {
    private enum Metrics {
        static let minimumHeight: CGFloat = 24
        static let collapsedMaximumHeight: CGFloat = 180
        static let editingMinimumHeight: CGFloat = 80
    }

    let description: String
    @Binding var descriptionText: String
    @Binding var editingDescription: Bool
    let canEditDescription: Bool
    let canShowDiffEditButton: Bool
    let onSave: (String) -> Void
    let onOpenDiffEdit: () -> Void

    @State private var contentHeight: CGFloat = 0
    @State private var isExpanded = false

    private var isEditingDescription: Bool {
        canEditDescription && editingDescription
    }

    private var minimumHeight: CGFloat {
        isEditingDescription ? Metrics.editingMinimumHeight : Metrics.minimumHeight
    }

    private var descriptionHeight: CGFloat {
        let height = max(contentHeight, minimumHeight)
        return isExpanded ? height : min(height, Metrics.collapsedMaximumHeight)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            descriptionHeader
            if isEditingDescription || !description.isEmpty {
                descriptionBody
            }
        }
        .layoutPriority(1)
    }

    private var descriptionHeader: some View {
        HStack(spacing: 8) {
            Text("Description")
                .jayjayFont(14, weight: .semibold)
            if isEditingDescription {
                Button("Save") {
                    onSave(descriptionText)
                    editingDescription = false
                }
                .keyboardShortcut("s")
                .controlSize(.small)
                Button("Cancel") {
                    descriptionText = description
                    editingDescription = false
                }
                .controlSize(.small)
            } else if canEditDescription {
                Button {
                    editingDescription = true
                } label: {
                    Label("Edit", systemImage: "pencil")
                        .labelStyle(.titleAndIcon)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Edit message")
            }
            if contentHeight > Metrics.collapsedMaximumHeight, isEditingDescription || !description.isEmpty {
                Button {
                    isExpanded.toggle()
                } label: {
                    Label(
                        isExpanded ? "Collapse description" : "Expand description",
                        systemImage: isExpanded ? "chevron.up" : "chevron.down"
                    )
                    .labelStyle(.iconOnly)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help(isExpanded ? "Collapse description" : "Expand description")
                .accessibilityIdentifier(AID.Detail.descriptionExpansion)
            }
            Spacer()
            if canShowDiffEditButton {
                Button("Edit Diff...") { onOpenDiffEdit() }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .accessibilityIdentifier(AID.DiffEdit.open)
                    .help("Open dedicated diff edit mode")
            }
        }
    }

    private var descriptionBody: some View {
        ScrollView {
            // The zero-width space measures the editor's last empty line, including the insertion point after a trailing newline.
            Text(isEditingDescription ? descriptionText + "\u{200B}" : description)
                .jayjayFont(13, design: .monospaced)
                .textSelection(.enabled)
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, isEditingDescription ? 5 : 0)
                .onGeometryChange(for: CGFloat.self) { geometry in
                    ceil(geometry.size.height)
                } action: { height in
                    contentHeight = height
                    if height <= Metrics.collapsedMaximumHeight {
                        isExpanded = false
                    }
                }
        }
        .opacity(isEditingDescription ? 0 : 1)
        .accessibilityHidden(isEditingDescription)
        .accessibilityIdentifier(AID.Detail.description)
        .overlay {
            if isEditingDescription {
                TextEditor(text: $descriptionText)
                    .jayjayFont(13, design: .monospaced)
                    .scrollContentBackground(.hidden)
                    .accessibilityIdentifier(AID.Detail.descriptionEditor)
            }
        }
        .frame(minHeight: minimumHeight, idealHeight: descriptionHeight, maxHeight: descriptionHeight)
        .padding(isEditingDescription ? 6 : 0)
        .background {
            if isEditingDescription {
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.primary.opacity(0.04))
                    .stroke(Color.primary.opacity(0.1))
            }
        }
    }
}
