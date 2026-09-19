import JayJayCore
import SwiftUI

extension ChangeDetailView {
    static func canEnterConflictEditor(info: ChangeInfo, hunk: DiffHunk, isCompareMode: Bool) -> Bool {
        !info.isImmutable && !isCompareMode && hunk.supportsConflictEditor
    }

    @ViewBuilder var metadataSection: some View {
        if descriptionExpanded.wrappedValue {
            metadataGrid
        } else {
            byline
        }
    }

    private var metadataGrid: some View {
        Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 8, verticalSpacing: 5) {
            IdentifierRow(label: "Author:") {
                author(showsEmail: true)
            }
            IdentifierRow(label: "Date:") {
                Text(formatTimestamp(detail.info.author.timestampMillis))
                    .jayjayFont(11)
                    .textSelection(.enabled)
            }
            IdentifierRow(label: "Change:") {
                IdentifierText(
                    value: detail.info.changeId.id,
                    prefixLength: Int(detail.info.changeId.shortLen),
                    prefixStyle: .changeId
                )
                CopyIconButton(value: detail.info.changeId.id, help: "Copy change")
            }
            IdentifierRow(label: "Commit:") {
                IdentifierText(
                    value: String(detail.info.commitId.id.prefix(12)),
                    prefixLength: Int(detail.info.commitId.shortLen)
                )
                CopyIconButton(value: detail.info.commitId.id, help: "Copy commit")
            }
            if !detail.info.parents.isEmpty {
                IdentifierRow(label: "Parents:") {
                    IdentifierText(value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
                }
            }
            if !detail.info.bookmarks.isEmpty {
                IdentifierRow(label: "Bookmarks:") {
                    ForEach(detail.info.bookmarks, id: \.self) { bookmarkChip($0, showsCopy: true) }
                }
            }
            if !detail.info.tags.isEmpty {
                IdentifierRow(label: "Tags:") {
                    ForEach(detail.info.tags, id: \.self) { tagChip($0) }
                }
            }
            if let visibleDiffStats {
                IdentifierRow(label: "Changes:") { diffStatsView(visibleDiffStats, spelledOut: true) }
            }
        }
    }

    private var byline: some View {
        ViewThatFits(in: .horizontal) {
            byline(showsDate: true)
            byline(showsDate: false)
        }
    }

    private func byline(showsDate: Bool) -> some View {
        HStack(spacing: 6) {
            author(showsEmail: false)
            bylineSeparator
            IdentifierText(
                value: String(detail.info.changeId.id.prefix(12)),
                prefixLength: Int(detail.info.changeId.shortLen),
                prefixStyle: .changeId
            )
            CopyIconButton(value: detail.info.changeId.id, help: "Copy change")
            if showsDate {
                bylineSeparator
                Text(formatTimestamp(detail.info.author.timestampMillis))
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            Spacer(minLength: 8)
            if let visibleDiffStats {
                diffStatsView(visibleDiffStats)
            }
            ForEach(detail.info.bookmarks, id: \.self) { bookmarkChip($0, showsCopy: false) }
            ForEach(detail.info.tags, id: \.self) { tagChip($0) }
        }
        .lineLimit(1)
    }

    private func author(showsEmail: Bool) -> some View {
        HStack(spacing: 5) {
            CommitAvatar(email: detail.info.author.email, size: 14)
            Text(detail.info.author.name)
                .jayjayFont(11)
            if showsEmail {
                Text("<\(detail.info.author.email)>")
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
        }
        .textSelection(.enabled)
        .help("\(detail.info.author.name) <\(detail.info.author.email)>")
    }

    private var visibleDiffStats: DiffStats? {
        guard let stats = diffStats, stats.insertions > 0 || stats.deletions > 0 else { return nil }
        return stats
    }

    private func diffStatsView(_ stats: DiffStats, spelledOut: Bool = false) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: spelledOut ? 8 : 4) {
            if stats.insertions > 0 {
                diffStat("+\(stats.insertions)", color: .green, caption: spelledOut ? "added" : nil, count: stats.insertions)
            }
            if stats.deletions > 0 {
                diffStat("-\(stats.deletions)", color: .red, caption: spelledOut ? "removed" : nil, count: stats.deletions)
            }
        }
        .accessibilityElement(children: .ignore)
        .accessibilityIdentifier(AID.Detail.diffStats(insertions: stats.insertions, deletions: stats.deletions))
    }

    private func diffStat(_ value: String, color: Color, caption: String?, count: UInt32) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 3) {
            Text(value)
                .jayjayFont(11, weight: .semibold, design: .monospaced)
                .foregroundStyle(color)
            if let caption {
                Text("\(caption) \(count == 1 ? "line" : "lines")")
                    .jayjayFont(11)
                    .foregroundStyle(color)
            }
        }
    }

    private var bylineSeparator: some View {
        Text("·")
            .jayjayFont(11)
            .foregroundStyle(.tertiary)
            .accessibilityHidden(true)
    }

    var compareBanner: some View {
        HStack(spacing: 8) {
            if compareDisplay?.isCombinedSelection == true {
                Image(systemName: "square.stack.3d.up.fill")
                    .foregroundStyle(.orange)
                    .accessibilityLabel("Combined selection")
                    .accessibilityIdentifier(AID.Compare.combinedSelection)
            } else {
                Button {
                    onReverseCompare?()
                } label: {
                    Image(systemName: "arrow.left.arrow.right")
                        .foregroundStyle(.orange)
                }
                .buttonStyle(.plain)
                .disabled(onReverseCompare == nil)
                .help("Reverse compare direction")
                .accessibilityIdentifier(AID.Compare.reverseDirection)
            }
            Text(compareDisplay?.title ?? "Comparing")
                .jayjayFont(12, weight: .medium)
                .accessibilityIdentifier(AID.Compare.banner)
            compareLabel(compareDisplay?.from ?? String(compareFromId?.prefix(8) ?? ""))
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            compareLabel(compareDisplay?.to ?? String(detailRevision.prefix(8)))
            Spacer()
            Text("\(detail.diff.count) files changed")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                onClearCompare?()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .help("Exit compare mode")
        }
        .padding(.horizontal, PaneLayout.detailInset)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.08), ignoresSafeAreaEdges: [])
    }

    private func tagChip(_ name: String) -> some View {
        HStack(spacing: 3) {
            Image(systemName: "tag")
                .jayjayFont(9, weight: .semibold)
                .foregroundStyle(.blue)
            Text(name).jayjayFont(11, design: .monospaced)
        }
        .padding(.horizontal, 6).padding(.vertical, 2)
        .background(Color.primary.opacity(0.08), in: .capsule)
        .lineLimit(1)
        .help("Tag: \(name)")
    }

    private func bookmarkChip(_ name: String, showsCopy: Bool) -> some View {
        let conflicted = conflictedBookmarkNames.contains(name)
        return HStack(spacing: 4) {
            HStack(spacing: 3) {
                if conflicted {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .jayjayFont(9, weight: .semibold)
                        .foregroundStyle(.orange)
                }
                Text(name).jayjayFont(11, design: .monospaced)
            }
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(
                conflicted ? Color.orange.opacity(0.18) : Color.accentColor.opacity(0.15),
                in: .capsule
            )
            if showsCopy {
                CopyIconButton(value: name, help: "Copy bookmark name")
            }
        }
        .lineLimit(1)
        .help(
            conflicted
                ? "Conflicted bookmark: \(name) — points at more than one change"
                : "Bookmark: \(name)"
        )
    }

    private func compareLabel(_ text: String) -> some View {
        Text(text)
            .jayjayFont(12, weight: .semibold, design: .monospaced)
            .lineLimit(1)
            .truncationMode(.middle)
    }

    func formatTimestamp(_ millis: Int64) -> String {
        Date(timeIntervalSince1970: Double(millis) / 1000.0).formatted(.dateTime.year().month().day().hour().minute())
    }

    func conflictBar(hunk: DiffHunk) -> some View {
        let path = hunk.path
        return HStack(spacing: 10) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.red)
            Text("Conflict")
                .jayjayFont(12, weight: .semibold)
            Spacer()
            Button("Use Ours") {
                actions?.resolveUseOurs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useOurs(path))
            Button("Use Theirs") {
                actions?.resolveUseTheirs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useTheirs(path))
            if Self.canEnterConflictEditor(info: detail.info, hunk: hunk, isCompareMode: isCompareMode) {
                Button("Edit in JayJay") {
                    prepareConflictEditor(path: path)
                }
                .buttonStyle(.bordered)
                .disabled(conflictEditorPreparation != nil)
                .accessibilityIdentifier(AID.Conflict.resolveInJayJay(path))
            }
            if let tool = appSettings.externalEditor.jjMergeTool {
                Button("Resolve in \(appSettings.externalEditor.title)") {
                    actions?.resolveInEditor(rev: detailRevision, path: path, tool: tool)
                }
                .buttonStyle(.bordered)
            } else {
                Button("Open in \(appSettings.externalEditor.title)") {
                    appSettings.openInEditor(filePath: path, repoPath: repoPath)
                }
                .buttonStyle(.bordered)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.red.opacity(0.08))
    }
}
