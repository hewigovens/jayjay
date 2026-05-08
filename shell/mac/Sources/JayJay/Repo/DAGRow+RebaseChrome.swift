import SwiftUI

extension DAGRow {
    @ViewBuilder
    var leadingAccent: some View {
        if let accent = viewModel.leadingAccentColor {
            RoundedRectangle(cornerRadius: 2, style: .continuous)
                .fill(accent)
                .frame(width: 3)
        }
    }

    @ViewBuilder
    var rebaseOutline: some View {
        switch viewModel.outlineState {
            case .hoverTarget?:
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(Color.accentColor, lineWidth: 2)
                    .padding(.vertical, 2)
            case .armed?:
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(
                        Color.accentColor.opacity(0.7),
                        style: StrokeStyle(lineWidth: 1.5, dash: [5, 4])
                    )
                    .padding(.vertical, 2)
            case nil:
                EmptyView()
        }
    }

    @ViewBuilder
    var rebaseBeforeGuide: some View {
        if viewModel.hoverPlacement == .before {
            rebaseInsertGuide
                .padding(.horizontal, 8)
        }
    }

    @ViewBuilder
    var rebaseAfterGuide: some View {
        if viewModel.hoverPlacement == .after {
            rebaseInsertGuide
                .padding(.horizontal, 8)
        }
    }

    @ViewBuilder
    var dragTargetBubbleOverlay: some View {
        if let dragTargetText = viewModel.dragTargetText {
            dragTargetBubble(dragTargetText)
                .padding(.trailing, 10)
        }
    }

    private func dragTargetBubble(_ text: String) -> some View {
        HStack(spacing: 6) {
            Text(text)
                .jayjayFont(10, weight: .medium)
                .lineLimit(1)
            if viewModel.showsReturnHint {
                hintChip("return")
            }
            hintChip("esc")
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            viewModel.isRebaseHoverTarget ? Color.accentColor.opacity(0.14) : Color.clear,
            in: Capsule()
        )
        .background(.regularMaterial, in: Capsule())
        .overlay(
            Capsule()
                .stroke(Color.accentColor.opacity(viewModel.isRebaseHoverTarget ? 0.5 : 0.2), lineWidth: 1)
        )
    }

    private var rebaseInsertGuide: some View {
        Capsule()
            .fill(Color.accentColor)
            .frame(height: 3)
            .shadow(color: .accentColor.opacity(0.35), radius: 4)
    }

    private func hintChip(_ text: String) -> some View {
        Text(text.uppercased())
            .jayjayFont(8, weight: .semibold, design: .monospaced)
            .foregroundStyle(.secondary)
            .padding(.horizontal, 4)
            .padding(.vertical, 2)
            .background(Color.primary.opacity(0.06), in: Capsule())
    }
}
