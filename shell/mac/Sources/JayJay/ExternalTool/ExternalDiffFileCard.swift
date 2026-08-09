import JayJayCore
import JayJayDiffUI
import SwiftUI

struct ExternalDiffFileCard: View, DiffGutterSelectionActions {
    @Bindable var file: ExternalDiffFileState
    let editable: Bool
    let onToggleFile: () -> Void

    @Environment(\.jayjayFontSize) private var fontSize
    @Environment(\.jayjayFontFamily) private var fontFamily

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            if !file.isCollapsed {
                bodyContent
            }
        }
        .padding(14)
        .background(Color.primary.opacity(0.025), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(selectionColor, lineWidth: 1)
        )
        .environment(\.diffFontSize, fontSize)
        .environment(\.diffFontFamily, fontFamily.nsFontName)
    }

    private var header: some View {
        HStack(spacing: 8) {
            Button {
                file.isCollapsed.toggle()
            } label: {
                Image(systemName: file.isCollapsed ? "chevron.right" : "chevron.down")
                    .frame(width: 12)
            }
            .buttonStyle(.plain)
            if editable {
                Button(action: onToggleFile) {
                    Image(systemName: selectionImage)
                        .foregroundStyle(file.keepsAnyChanges ? Color.accentColor : Color.secondary.opacity(0.5))
                }
                .buttonStyle(.plain)
                .help("Keep or discard every change in this file")
                .accessibilityIdentifier(AID.ExternalTool.fileToggle(file.hunk.path))
            }
            Image(systemName: file.hunk.hunkType.iconName)
                .foregroundStyle(file.hunk.hunkType.iconColor)
            Text(file.hunk.path)
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .textSelection(.enabled)
            stats
            HStack(spacing: 8) {
                Spacer(minLength: 12)
                if editable, !file.supportsEditing {
                    Text("Whole-file selection")
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                }
            }
        }
        .contentShape(Rectangle())
        .onTapGesture { file.isCollapsed.toggle() }
    }

    @ViewBuilder
    private var bodyContent: some View {
        if file.hunk.old.preview != nil || file.hunk.new.preview != nil {
            ImageDiffView(
                oldPath: file.hunk.old.preview?.imagePath,
                newPath: file.hunk.new.preview?.imagePath,
                hunkType: file.hunk.hunkType
            )
            .frame(height: 320)
        } else if !file.displayDiff.lines.isEmpty {
            NativeDiffView(
                diff: file.displayDiff,
                gutterActions: editable && file.supportsEditing ? self : nil,
                contentGeneration: UInt64(bitPattern: Int64(file.selectedLines.hashValue)),
                onContentHeightChanged: { height in
                    if abs((file.measuredHeight ?? 0) - height) > 0.5 {
                        file.measuredHeight = height
                    }
                }
            )
            .frame(height: file.measuredHeight ?? estimatedHeight)
        } else {
            Text("The file contents differ, but no inline textual preview is available.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, minHeight: 100)
        }
    }

    private var stats: some View {
        HStack(spacing: 4) {
            if file.stats.insertions > 0 {
                Text("+\(file.stats.insertions)").foregroundStyle(.green)
            }
            if file.stats.deletions > 0 {
                Text("-\(file.stats.deletions)").foregroundStyle(.red)
            }
            if file.executableChanged {
                Text("mode").foregroundStyle(.secondary)
            }
        }
        .jayjayFont(11, weight: .semibold, design: .monospaced)
    }

    private var selectionImage: String {
        if !file.keepsAnyChanges {
            return "circle"
        }
        if file.keepsAllChanges {
            return "checkmark.circle.fill"
        }
        return "minus.circle.fill"
    }

    private var selectionColor: Color {
        editable && file.keepsAnyChanges ? Color.accentColor.opacity(0.35) : Color.primary.opacity(0.08)
    }

    private var estimatedHeight: CGFloat {
        max(CGFloat(max(file.displayDiff.lines.count, 1)) * max(18, fontSize + 5) + 24, 44)
    }

    var currentSelectedLineRange: ClosedRange<Int>? {
        nil
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}
    func selectFile() {
        file.selectSide(.new)
    }

    func selectChangeGroup(_ lineRange: ClosedRange<Int>) {
        file.selectDisplayRange(lineRange)
    }

    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState? {
        guard let fullLine = file.displayToFull[lineNumber] else { return nil }
        return file.selectedLines.contains(fullLine) ? .selected : .unselected
    }

    func toggleLineCheckbox(_ lineNumber: Int) {
        file.toggleDisplayLine(lineNumber)
    }
}
