import JayJayCore
import SwiftUI

extension ChangeDetailView {
    static func canEnterConflictEditor(info: ChangeInfo, hunk: DiffHunk, isCompareMode: Bool) -> Bool {
        !info.isImmutable && !isCompareMode && hunk.supportsConflictEditor
    }

    var headerSection: some View {
        VStack(alignment: .leading, spacing: 6) {
            CopyableRow(
                "Change",
                value: detail.info.changeId.id,
                emphasizedPrefix: Int(detail.info.changeId.shortLen)
            )
            CopyableRow(
                "Commit",
                value: String(detail.info.commitId.id.prefix(12)),
                copyValue: detail.info.commitId.id,
                emphasizedPrefix: Int(detail.info.commitId.shortLen)
            )
            HStack(spacing: 6) {
                Text("Author").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                CommitAvatar(email: detail.info.author.email, size: 18)
                Text("\(detail.info.author.name) <\(detail.info.author.email)>")
                    .jayjayFont(11, design: .monospaced)
                    .textSelection(.enabled)
            }
            LabeledRow("Date", value: formatTimestamp(detail.info.author.timestampMillis))
            if !detail.info.parents.isEmpty {
                LabeledRow("Parents", value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
            }
            if !detail.info.bookmarks.isEmpty {
                HStack(spacing: 4) {
                    Text("Bookmarks").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    ForEach(detail.info.bookmarks, id: \.self) { name in
                        HStack(spacing: 4) {
                            Text(name).jayjayFont(11, design: .monospaced)
                                .padding(.horizontal, 6).padding(.vertical, 2)
                                .background(.tint.opacity(0.15), in: .capsule)
                            CopyIconButton(value: name, help: "Copy bookmark name")
                        }
                    }
                }
            }
            if let stats = diffStats, stats.insertions > 0 || stats.deletions > 0 {
                HStack(spacing: 4) {
                    Text("Changes").jayjayFont(11).foregroundStyle(.secondary).frame(width: 70, alignment: .trailing)
                    if stats.insertions > 0 {
                        Text("+\(stats.insertions)")
                            .jayjayFont(11, weight: .semibold, design: .monospaced)
                            .foregroundStyle(.green)
                    }
                    if stats.deletions > 0 {
                        Text("-\(stats.deletions)")
                            .jayjayFont(11, weight: .semibold, design: .monospaced)
                            .foregroundStyle(.red)
                    }
                }
                .accessibilityElement(children: .ignore)
                .accessibilityIdentifier(AID.Detail.diffStats(insertions: stats.insertions, deletions: stats.deletions))
            }
        }
    }

    var compareBanner: some View {
        HStack(spacing: 8) {
            Button {
                onReverseCompare?()
            } label: {
                Image(systemName: "arrow.left.arrow.right")
                    .foregroundStyle(.orange)
            }
            .buttonStyle(.plain)
            .help("Reverse compare direction")
            Text(compareDisplay?.title ?? "Comparing")
                .jayjayFont(12, weight: .medium)
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
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.08))
        .accessibilityIdentifier(AID.Compare.banner)
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
