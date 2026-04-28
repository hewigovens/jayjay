import SwiftUI

extension ChangeDetailView {
    var descriptionSection: some View {
        DetailDescriptionSection(
            description: detail.info.description,
            descriptionText: $descriptionText,
            editingDescription: $editingDescription,
            canShowDiffEditButton: canShowDiffEditButton,
            onSave: { onDescribe(detail.info.changeId, $0) },
            onOpenDiffEdit: { paneMode = .diffEdit }
        )
        .id("\(detail.info.changeId)|\(detail.info.commitId)")
    }

    private var canShowDiffEditButton: Bool {
        !isCompareMode && !detail.info.hasConflict && !detail.diff.isEmpty && !editingDescription
    }
}

private struct DetailDescriptionSection: View {
    private enum Metrics {
        static let compactHeight: CGFloat = 32
        static let minimumHeight: CGFloat = 24
        static let maximumHeight: CGFloat = 180
        static let editingMinimumHeight: CGFloat = 80
    }

    let description: String
    @Binding var descriptionText: String
    @Binding var editingDescription: Bool
    let canShowDiffEditButton: Bool
    let onSave: (String) -> Void
    let onOpenDiffEdit: () -> Void

    @State private var descriptionHeight: CGFloat = Metrics.compactHeight
    @GestureState private var resizeTranslation: CGFloat = 0

    private var visibleDescriptionHeight: CGFloat {
        let minimum = editingDescription ? Metrics.editingMinimumHeight : Metrics.minimumHeight
        let baseHeight = editingDescription ? max(descriptionHeight, Metrics.editingMinimumHeight) : descriptionHeight
        return clampedDescriptionHeight(baseHeight, minimum: minimum)
    }

    private var previewDescriptionHeight: CGFloat {
        clampedDescriptionHeight(visibleDescriptionHeight + resizeTranslation, minimum: minimumDescriptionHeight)
    }

    private var minimumDescriptionHeight: CGFloat {
        editingDescription ? Metrics.editingMinimumHeight : Metrics.minimumHeight
    }

    private var resizePreviewOffset: CGFloat {
        previewDescriptionHeight - visibleDescriptionHeight
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            descriptionHeader
            descriptionBody
        }
    }

    private var descriptionHeader: some View {
        HStack(spacing: 8) {
            Text("Description")
                .jayjayFont(14, weight: .semibold)
            if editingDescription {
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
            } else {
                Button {
                    editingDescription = true
                    descriptionHeight = max(descriptionHeight, Metrics.editingMinimumHeight)
                } label: {
                    Label("Edit", systemImage: "pencil")
                        .labelStyle(.titleAndIcon)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Edit message")
            }
            Spacer()
            if canShowDiffEditButton {
                Button("Edit Diff...") { onOpenDiffEdit() }
                    .buttonStyle(.bordered)
                    .controlSize(.small)
                    .help("Open dedicated diff edit mode")
            }
        }
    }

    @ViewBuilder
    private var descriptionBody: some View {
        if editingDescription {
            VStack(spacing: 2) {
                TextEditor(text: $descriptionText)
                    .jayjayFont(13, design: .monospaced)
                    .frame(height: visibleDescriptionHeight)
                    .scrollContentBackground(.hidden)
                    .padding(6)
                    .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8))
                    .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.primary.opacity(0.1)))
                descriptionResizeHandle
            }
        } else if !description.isEmpty {
            VStack(spacing: 2) {
                ScrollView {
                    Text(description)
                        .jayjayFont(13, design: .monospaced)
                        .textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(height: visibleDescriptionHeight)
                descriptionResizeHandle
            }
        }
    }

    private var descriptionResizeHandle: some View {
        ZStack {
            Capsule()
                .fill(Color.accentColor.opacity(resizeTranslation == 0 ? 0 : 0.45))
                .frame(height: 2)
                .padding(.horizontal, 20)
                .offset(y: resizePreviewOffset)
                .opacity(resizeTranslation == 0 ? 0 : 1)

            Capsule()
                .fill(Color.secondary.opacity(0.35))
                .frame(width: 36, height: 3)
                .offset(y: resizePreviewOffset)
        }
        .frame(maxWidth: .infinity, minHeight: 10)
        .contentShape(Rectangle())
        .gesture(descriptionResizeGesture)
        .help("Resize description")
    }

    private var descriptionResizeGesture: some Gesture {
        DragGesture(minimumDistance: 1)
            .updating($resizeTranslation) { value, state, transaction in
                transaction.disablesAnimations = true
                state = value.translation.height
            }
            .onEnded { value in
                var transaction = Transaction()
                transaction.disablesAnimations = true
                withTransaction(transaction) {
                    descriptionHeight = clampedDescriptionHeight(
                        visibleDescriptionHeight + value.translation.height,
                        minimum: minimumDescriptionHeight
                    )
                }
            }
    }

    private func clampedDescriptionHeight(_ height: CGFloat, minimum: CGFloat) -> CGFloat {
        min(max(height, minimum), Metrics.maximumHeight)
    }
}
