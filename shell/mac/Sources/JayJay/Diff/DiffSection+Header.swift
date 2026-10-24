import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffSection {
    var diffHeader: some View {
        HStack(alignment: .firstTextBaseline, spacing: 8) {
            FileIcon(path: hunk.path)
            renamePathLabel
            FilePathLabel(path: hunk.path, size: 13)
                .textSelection(.enabled)
                .help(hunk.path)
                .accessibilityIdentifier(AID.Diff.section)
            CopyIconButton(value: hunk.path, help: Localized.Diff.copyPath)
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
                        actionLabel(Localized.Diff.editDiff, systemImage: "square.and.pencil")
                    }
                    .keyboardFocusStop(.editDiff, action: openDiffEdit)
                    .help(Localized.Diff.openDedicatedDiffEditMode)
                    .accessibilityLabel(Localized.Diff.editDiff)
                    .accessibilityIdentifier(AID.DiffEdit.open)
                }
                if canEditFile {
                    Button(action: editFile) {
                        actionLabel(Localized.Diff.editFile, systemImage: "pencil")
                    }
                    .keyboardFocusStop(.editFile, action: editFile)
                    .help(Localized.Diff.editThisWorkingCopyFile)
                    .accessibilityLabel(Localized.Diff.editFile)
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

    private func editFile() {
        onEditFile?(nil)
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
                inactiveHelp: Localized.Diff.showRenderedSvg
            ) {
                toggleSvgRichView()
            }
        }
        if canRenderMarkdownFilePreview {
            richPreviewButton(
                icon: activeMarkdownRichView ? "eye.fill" : "eye",
                active: activeMarkdownRichView,
                inactiveHelp: Localized.Diff.showRenderedMarkdown
            ) {
                toggleMarkdownRichView()
            }
        }
        if canOpenHTMLExternally {
            richPreviewButton(
                icon: activeHTMLRichView ? "eye.fill" : "eye",
                active: activeHTMLRichView,
                inactiveHelp: htmlPreviewMayNeedScript
                    ? Localized.Diff.showRenderedHtmlScriptsWontRun
                    : Localized.Diff.showRenderedHtml
            ) {
                toggleHTMLRichView()
            }
            externalOpenButton(
                icon: "arrow.up.right.square",
                help: Localized.Diff.openWorkingCopyHtmlInDefault
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
            actionLabel(Localized.Common.preview, systemImage: icon)
        }
        .accessibilityIdentifier(AID.Diff.preview)
        .buttonStyle(HeaderActionButtonStyle(isActive: active))
        .help(active ? Localized.Diff.showSourceDiff : inactiveHelp)
    }

    @ViewBuilder
    private var renamePathLabel: some View {
        if hunk.hunkType.hasSource, let oldPath = hunk.oldPath {
            Text(oldPath)
                .jayjayFont(11, design: .monospaced)
                .strikethrough(hunk.hunkType == .renamed)
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
                effectiveSideBySideDiff ? Localized.Diff.sideBySide : Localized.Diff.unified,
                systemImage: effectiveSideBySideDiff ? "rectangle.split.2x1" : "text.justify"
            )
        }
        .keyboardFocusStop(.diffLayout, action: toggle)
        .help(effectiveSideBySideDiff ? Localized.Diff.switchToUnified : Localized.Diff.switchToSideBySide)
        .accessibilityLabel(effectiveSideBySideDiff ? Localized.Diff.sideBySide : Localized.Diff.unified)
        .accessibilityIdentifier(AID.Diff.layoutToggle)
    }

    private var effectiveSideBySideDiff: Bool {
        guard settings.sideBySideDiff else { return false }
        guard let fileDiff else { return true }
        return canUseSideBySide(fileDiff)
    }
}
