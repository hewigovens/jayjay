import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection {
    var diffHeader: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            renamePathLabel
            FilePathLabel(path: hunk.path, size: 13)
                .textSelection(.enabled)
                .help(hunk.path)
                .accessibilityIdentifier(AID.Diff.section)
            CopyIconButton(value: hunk.path, help: "Copy path")
            Text(hunk.hunkType.label)
                .jayjayFont(10, weight: .medium)
                .foregroundStyle(hunk.hunkType.iconColor)
                .padding(.horizontal, 7)
                .padding(.vertical, 1)
                .background(hunk.hunkType.iconColor.opacity(0.12), in: Capsule())
                .fixedSize()
            if let lineStats, lineStats.hasLineChanges {
                LineStatsLabel(stats: lineStats)
            }
            Spacer(minLength: 8)
            HStack(spacing: 2) {
                richPreviewButtons
                sideBySideButton
                if canOpenDiffEdit {
                    Button(action: openDiffEdit) {
                        actionLabel("Edit Diff", systemImage: "square.and.pencil")
                    }
                    .keyboardFocusStop(.editDiff, action: openDiffEdit)
                    .help("Open dedicated diff edit mode")
                    .accessibilityLabel("Edit Diff")
                    .accessibilityIdentifier(AID.DiffEdit.open)
                }
                if let onEditFile, canEditLoadedWorkingCopyFile {
                    Button(action: onEditFile) {
                        actionLabel("Edit File", systemImage: "pencil")
                    }
                    .keyboardFocusStop(.editFile, action: onEditFile)
                    .help("Edit this working-copy file")
                    .accessibilityLabel("Edit File")
                    .accessibilityIdentifier(AID.FileEditor.open(hunk.path))
                }
            }
            .buttonStyle(HeaderActionButtonStyle())
            .fixedSize()
        }
        .onGeometryChange(for: Bool.self) { $0.size.width < Self.compactHeaderWidth } action: { compactHeader = $0 }
    }

    private func actionLabel(_ title: String, systemImage: String) -> some View {
        HStack(spacing: 4) {
            Image(systemName: systemImage)
            if !compactHeader {
                Text(title)
            }
        }
    }

    /// Below this the labelled controls would squeeze the file name to nothing, so they drop to icons.
    private static let compactHeaderWidth: CGFloat = 600

    private var canEditLoadedWorkingCopyFile: Bool {
        guard hasCurrentRenderableDiff,
              loadedDiff?.content.supportsFileEditor == true
        else { return false }
        return true
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
            richPreviewButton(
                icon: activeHTMLRichView ? "eye.fill" : "eye",
                active: activeHTMLRichView,
                inactiveHelp: htmlPreviewMayNeedScript
                    ? "Show rendered HTML (scripts won't run in this preview)"
                    : "Show rendered HTML"
            ) {
                toggleHTMLRichView()
            }
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
        }
        .help(help)
    }

    private func richPreviewButton(
        icon: String,
        active: Bool,
        inactiveHelp: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            actionLabel("Preview", systemImage: icon)
        }
        .buttonStyle(HeaderActionButtonStyle(isActive: active))
        .help(active ? "Show source diff" : inactiveHelp)
    }

    @ViewBuilder
    private var renamePathLabel: some View {
        if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
            Text(oldPath)
                .jayjayFont(11, design: .monospaced)
                .strikethrough()
                .lineLimit(1)
                .truncationMode(.middle)
                .help(oldPath)
                .foregroundStyle(.secondary)
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
        }
    }

    private var sideBySideButton: some View {
        let toggle = { settings.sideBySideDiff.toggle() }
        return Button(action: toggle) {
            actionLabel(
                effectiveSideBySideDiff ? "Side-by-side" : "Unified",
                systemImage: effectiveSideBySideDiff ? "rectangle.split.2x1" : "text.justify"
            )
        }
        .keyboardFocusStop(.diffLayout, action: toggle)
        .help(effectiveSideBySideDiff ? "Switch to unified" : "Switch to side-by-side")
        .accessibilityLabel(effectiveSideBySideDiff ? "Side-by-side" : "Unified")
    }

    private var effectiveSideBySideDiff: Bool {
        guard settings.sideBySideDiff else { return false }
        guard let fileDiff else { return true }
        return canUseSideBySide(fileDiff)
    }
}
