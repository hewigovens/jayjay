import JayJayCore
import SwiftUI

extension DiffSection {
    var diffHeader: some View {
        HStack {
            Image(systemName: hunk.hunkType.iconName)
                .foregroundStyle(hunk.hunkType.iconColor)
            Text(hunk.path)
                .jayjayFont(14, weight: .semibold, design: .monospaced)
                .lineLimit(1)
                .truncationMode(.middle)
                .textSelection(.enabled)
                .help(hunk.path)
            CopyIconButton(value: hunk.path, help: "Copy path")
            richPreviewButtons
            Spacer()
            renamePathLabel
            sideBySideButton
            Text(hunk.hunkType.label)
                .jayjayFont(11, weight: .semibold)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(hunk.hunkType.iconColor.opacity(0.12), in: Capsule())
        }
    }

    @ViewBuilder
    private var richPreviewButtons: some View {
        if shouldShowProjectionToggle {
            richPreviewButton(
                icon: DiffProjectionDisplayPolicy.iconName(for: effectiveProjection),
                active: activeProjectionRichView,
                inactiveHelp: DiffProjectionDisplayPolicy.help(for: effectiveProjection)
            ) {
                toggleProjectionRichView()
            }
        }
        if isSvgFile {
            richPreviewButton(
                icon: activeSvgRichView ? "eye.fill" : "eye",
                active: activeSvgRichView,
                inactiveHelp: "Show rendered SVG"
            ) {
                toggleSvgRichView()
            }
        }
        if canRenderMarkdownFilePreview {
            richPreviewButton(
                icon: activeMarkdownRichView ? "eye.fill" : "eye",
                active: activeMarkdownRichView,
                inactiveHelp: "Show rendered Markdown"
            ) {
                toggleMarkdownRichView()
            }
        }
        if canOpenHTMLExternally {
            externalOpenButton(
                icon: "arrow.up.right.square",
                help: "Open working-copy HTML in default app"
            ) {
                openHTMLExternally()
            }
        }
    }

    private func externalOpenButton(
        icon: String,
        help: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
        }
        .buttonStyle(.plain)
        .help(help)
    }

    private func richPreviewButton(
        icon: String,
        active: Bool,
        inactiveHelp: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: icon)
                .jayjayFont(11)
                .foregroundStyle(active ? Color.accentColor : .secondary)
        }
        .buttonStyle(.plain)
        .help(active ? "Show source diff" : inactiveHelp)
    }

    @ViewBuilder
    private var renamePathLabel: some View {
        if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
            Text(oldPath)
                .jayjayFont(11, design: .monospaced)
                .strikethrough()
                .foregroundStyle(.secondary)
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
        }
    }

    private var sideBySideButton: some View {
        Button {
            settings.sideBySideDiff.toggle()
        } label: {
            HStack(spacing: 5) {
                Image(
                    systemName: effectiveSideBySideDiff
                        ? "rectangle.split.2x1"
                        : "text.justify"
                )
                .jayjayFont(11)
                Text(effectiveSideBySideDiff ? "Side-by-side" : "Unified")
                    .jayjayFont(11)
            }
            .foregroundStyle(effectiveSideBySideDiff ? Color.accentColor : .secondary)
            .padding(.horizontal, 8)
            .padding(.vertical, 3)
            .background(
                effectiveSideBySideDiff
                    ? AnyShapeStyle(Color.accentColor.opacity(0.14))
                    : AnyShapeStyle(Color.primary.opacity(0.06)),
                in: RoundedRectangle(cornerRadius: 4, style: .continuous)
            )
        }
        .buttonStyle(.plain)
        .help(effectiveSideBySideDiff ? "Switch to unified" : "Switch to side-by-side")
    }

    private var effectiveSideBySideDiff: Bool {
        guard settings.sideBySideDiff else { return false }
        guard let fileDiff else { return true }
        return canUseSideBySide(fileDiff)
    }
}
