import JayJayCore
import SwiftUI

struct DiffEditFileSection: View {
    let hunk: DiffHunk
    let rev: String
    let repo: JayJayRepo?
    @Binding var selectionMode: DiffEditSelectionMode?
    let onLoaded: (DiffEditLoadedFile) -> Void

    @State private var fileDiff: FileDiff?
    @State private var oldContent: String?
    @State private var newContent: String?
    @State private var loadError: String?
    @State private var isLoading = false

    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            header
            content
        }
        .padding(14)
        .background(Color.primary.opacity(0.025), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .stroke(selectionMode == nil ? Color.primary.opacity(0.08) : Color.accentColor.opacity(0.35), lineWidth: 1)
        )
        .task(id: "\(rev)|\(hunk.path)|\(settings.ignoreWhitespace)") {
            await loadDiff()
        }
    }

    private var header: some View {
        HStack(spacing: 8) {
            Image(systemName: iconName(for: hunk.hunkType))
                .foregroundStyle(iconColor(for: hunk.hunkType))
            Text(hunk.path)
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .textSelection(.enabled)
            if let selectionMode {
                Text(selectionMode.badgeText)
                    .jayjayFont(10, weight: .semibold)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.accentColor.opacity(0.14), in: Capsule())
            }
            Spacer()
            if supportsDiffEdit {
                Button("Select File") { selectionMode = .file }
                    .buttonStyle(.borderless)
                if selectionMode != nil {
                    Button("Clear") { selectionMode = nil }
                        .buttonStyle(.borderless)
                }
            } else {
                Text("Text edits not supported")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
        }
    }

    @ViewBuilder
    private var content: some View {
        if isLoading {
            ProgressView()
                .frame(maxWidth: .infinity, minHeight: 120)
        } else if let loadError {
            Text(loadError)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, alignment: .leading)
        } else if let fileDiff {
            NativeDiffView(
                diff: fileDiff,
                gutterActions: supportsDiffEdit
                    ? DiffGutterContextActions(
                        splitFile: nil,
                        moveToWorkingCopy: nil,
                        restoreFile: nil,
                        abandonChange: nil,
                        openDiffEdit: nil,
                        selectFile: { selectionMode = .file },
                        selectHunk: { selectionMode = .hunk($0) },
                        onLineSelectionChanged: { selectionMode = .lines($0) },
                        selectedLineRange: selectionMode?.selectedLineRange
                    ) : nil
            )
            .frame(height: diffHeight(for: fileDiff))
        } else {
            Text("No textual preview available for this file.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, minHeight: 120)
        }
    }

    private var supportsDiffEdit: Bool {
        hunk.hunkType != .renamed && editableText(oldContent) && editableText(newContent)
    }

    private func loadDiff() async {
        guard let repo else { return }

        isLoading = true
        loadError = nil

        let loaded = await loadFileContent(repo: repo)
        let old = loaded.0
        let new = loaded.1
        let ignoreWhitespace = settings.ignoreWhitespace

        let diff = await Task.detached {
            repo.computeNativeDiffFull(
                path: hunk.path,
                oldContent: old ?? "",
                newContent: new ?? "",
                ignoreWhitespace: ignoreWhitespace
            )
        }.value

        await MainActor.run {
            oldContent = old
            newContent = new
            fileDiff = diff
            isLoading = false
            onLoaded(DiffEditLoadedFile(hunk: hunk, oldContent: old, newContent: new, diff: diff))
        }
    }

    private func loadFileContent(repo: JayJayRepo) async -> (String?, String?) {
        if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
            let renamedHunk = await Task.detached {
                try? repo.showFileRename(rev: rev, oldPath: oldPath, newPath: hunk.path)
            }.value
            return (renamedHunk?.oldContent, renamedHunk?.newContent)
        }

        let loaded = await Task.detached {
            try? repo.showFile(rev: rev, path: hunk.path)
        }.value
        if let loaded {
            let old = loaded.oldContent
            let new = loaded.newContent
            if old != nil || new != nil {
                return (old, new)
            }
        }

        let content = await Task.detached {
            try? repo.fileContent(rev: rev, path: hunk.path)
        }.value
        return (nil, content)
    }

    private func diffHeight(for diff: FileDiff) -> CGFloat {
        let lineHeight = max(18, CGFloat(settings.fontSize) + 5)
        return min(max(CGFloat(max(diff.lines.count, 4)) * lineHeight + 24, 120), 680)
    }

    private func editableText(_ text: String?) -> Bool {
        guard let text else { return true }
        return !text.hasPrefix("<binary file")
            && !text.hasPrefix("<directory>")
            && !text.hasPrefix("<git submodule")
            && !text.hasPrefix("<conflict")
            && !text.hasPrefix("<access denied")
    }

    private func iconName(for type: HunkType) -> String {
        switch type {
            case .added: "plus.circle.fill"
            case .removed: "minus.circle.fill"
            case .modified: "pencil.circle.fill"
            case .renamed: "arrow.right.circle.fill"
        }
    }

    private func iconColor(for type: HunkType) -> Color {
        switch type {
            case .added: .green
            case .removed: .red
            case .modified: .orange
            case .renamed: .blue
        }
    }
}
