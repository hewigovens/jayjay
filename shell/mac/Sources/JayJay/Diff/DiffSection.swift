import SwiftUI
import JayJayBindings

struct DiffSection: View {
    let hunk: DiffHunk
    let rev: String?
    let repo: JayJayRepo?

    @State private var fileDiff: FileDiff?
    @State private var isComputing = false
    @State private var loadedPath: String?
    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            diffHeader
            diffContent
        }
        .task(id: "\(rev ?? "")|\(hunk.path)|\(settings.ignoreWhitespace)") {
            await computeDiffAsync()
        }
    }

    private var diffHeader: some View {
        HStack {
            Image(systemName: iconName(for: hunk.hunkType))
                .foregroundStyle(iconColor(for: hunk.hunkType))
            Text(hunk.path)
                .jayjayFont(14, weight: .semibold, design: .monospaced)
                .textSelection(.enabled)
            Spacer()
            if hunk.hunkType == .renamed, let oldPath = hunk.oldPath {
                Text(oldPath)
                    .jayjayFont(11, design: .monospaced)
                    .strikethrough()
                    .foregroundStyle(.secondary)
                Image(systemName: "arrow.right")
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
            }
            Text(label(for: hunk.hunkType))
                .jayjayFont(11, weight: .semibold)
                .padding(.horizontal, 8)
                .padding(.vertical, 4)
                .background(iconColor(for: hunk.hunkType).opacity(0.12), in: Capsule())
        }
    }

    @ViewBuilder
    private var diffContent: some View {
        if isComputing {
            ProgressView()
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let diff = fileDiff, !diff.lines.isEmpty {
            Group {
                if settings.sideBySideDiff && isTwoColumnDiff(diff) {
                    SideBySideDiffView(diff: diff)
                        .id("sbs-\(hunk.path)")
                } else {
                    NativeDiffView(diff: diff)
                        .id("unified-\(hunk.path)")
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 12, style: .continuous)
                    .stroke(Color.primary.opacity(0.08), lineWidth: 1)
            )
        } else if hunk.oldContent == nil && hunk.newContent == nil && !isComputing && loadedPath == hunk.path {
            Text("No textual preview available for this file.")
                .foregroundStyle(.secondary)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
        }
    }

    private func isTwoColumnDiff(_ diff: FileDiff) -> Bool {
        let hasAdded = diff.lines.contains { $0.style == .added }
        let hasRemoved = diff.lines.contains { $0.style == .removed }
        return hasAdded && hasRemoved
    }

    private func computeDiffAsync() async {
        guard let repo else { return }

        let path = hunk.path
        let currentRev = rev

        // Check cache first
        if let cached = Self.cache[Self.cacheKey(rev: currentRev, path: path)] {
            fileDiff = cached
            loadedPath = path
            return
        }

        isComputing = true
        fileDiff = nil

        var old = hunk.oldContent ?? ""
        var new = hunk.newContent ?? ""

        // Lazy load content if not provided
        if old.isEmpty && new.isEmpty && hunk.hunkType != .renamed {
            if let currentRev {
                let fileHunk = await Task.detached {
                    try? repo.showFile(rev: currentRev, path: path)
                }.value

                // Staleness check: did the user switch to a different file?
                guard hunk.path == path else { return }

                old = fileHunk?.oldContent ?? ""
                new = fileHunk?.newContent ?? ""
            }
        }

        let ignoreWS = settings.ignoreWhitespace
        let result = await Task.detached {
            repo.computeNativeDiff(path: path, oldContent: old, newContent: new, ignoreWhitespace: ignoreWS)
        }.value

        // Staleness check again after diff computation
        guard hunk.path == path else { return }

        // Cache the result
        Self.cache[Self.cacheKey(rev: currentRev, path: path)] = result

        fileDiff = result
        loadedPath = path
        isComputing = false
    }

    // MARK: - Cache

    // Simple in-memory cache keyed by rev|path. Cleared when rev changes.
    private static var cache: [String: FileDiff] = [:]
    private static var cachedRev: String?

    private static func cacheKey(rev: String?, path: String) -> String {
        "\(rev ?? "")|\(path)"
    }

    static func clearCache() {
        cache.removeAll()
        cachedRev = nil
    }

    // MARK: - Helpers

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

    private func label(for type: HunkType) -> String {
        switch type {
        case .added: "Added"
        case .removed: "Removed"
        case .modified: "Modified"
        case .renamed: "Renamed"
        }
    }
}
