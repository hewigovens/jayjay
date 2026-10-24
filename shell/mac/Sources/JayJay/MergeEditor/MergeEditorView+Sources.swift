import JayJayCore
import SwiftUI

extension MergeEditorView {
    var sourcesPane: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 8) {
                Text(Localized.Repo.sources)
                    .jayjayFont(12, weight: .semibold)
                Text(Localized.Repo.useACompleteSideAsThe)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                Spacer()
                Button {
                    showsBase.toggle()
                } label: {
                    Label(
                        showsBase ? Localized.Merge.backToLeftRight : Localized.Merge.showBase,
                        systemImage: showsBase ? "arrow.uturn.backward" : "eye"
                    )
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
                .accessibilityIdentifier(AID.ExternalTool.baseVisibility)
            }
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            Divider()
            sourcePanes
        }
    }

    @ViewBuilder
    private var sourcePanes: some View {
        if showsBase {
            sourcePane(.base, content: session.base)
        } else {
            HSplitView {
                sourcePane(.left, content: session.left)
                sourcePane(.right, content: session.right)
            }
        }
    }

    private func sourcePane(_ source: MergeHunkSource, content: String) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(source.label)
                    .jayjayFont(12, weight: .semibold)
                Spacer()
                Button(Localized.Merge.useSource(source: source.label)) { session.useSource(source) }
                    .buttonStyle(.plain)
                    .jayjayFont(11, weight: .medium)
                    .foregroundStyle(.secondary)
                    .disabled(!session.canUseSources)
                    .accessibilityIdentifier(AID.ExternalTool.useSource(source.label.lowercased()))
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 7)
            Divider()
            CodeTextView(
                path: session.path,
                text: .constant(content.isEmpty ? "∅" : content),
                isEditable: false,
                wrapsLines: true,
                accessibilityIdentifier: AID.Conflict.editorSource(source.label.lowercased()),
                preparedText: content,
                preparedHighlightedLines: sourceHighlights(source)?.spans,
                preparedLineStyles: sourceHighlights(source)?.lineStyles,
                mergeScroll: scroll,
                mergePane: source.pane
            )
        }
    }

    private func sourceHighlights(_ source: MergeHunkSource) -> MergeSourceHighlights? {
        switch source {
            case .left: session.highlights?.left
            case .base: session.highlights?.base
            case .right: session.highlights?.right
        }
    }
}
