import JayJayCore
import SwiftUI

extension ChangeDetailView {
    static func canEnterConflictEditor(info: ChangeInfo, hunk: DiffHunk, isCompareMode: Bool) -> Bool {
        !info.isImmutable && !isCompareMode && hunk.supportsConflictEditor
    }

    @ViewBuilder var metadataSection: some View {
        if descriptionExpanded.wrappedValue {
            metadataGrid
                .matchedGeometryEffect(id: "metadata", in: metadataAnimation, properties: .position, anchor: .topLeading)
                .transition(.identity)
        } else {
            byline
                .matchedGeometryEffect(id: "metadata", in: metadataAnimation, properties: .position, anchor: .topLeading)
                .transition(.identity)
        }
    }

    private var metadataGrid: some View {
        Grid(alignment: .leadingFirstTextBaseline, horizontalSpacing: 8, verticalSpacing: 5) {
            IdentifierRow(label: Localized.Repo.authorColon) {
                person(detail.info.author)
            }
            if !detail.info.coAuthors.isEmpty {
                IdentifierRow(label: Localized.Repo.coAuthorsColon) {
                    VStack(alignment: .leading, spacing: 5) {
                        ForEach(Array(detail.info.coAuthors.enumerated()), id: \.offset) { person($0.element) }
                    }
                }
            }
            IdentifierRow(label: Localized.Repo.dateColon) {
                Text(formatTimestamp(detail.info.author.timestampMillis))
                    .jayjayFont(11)
                    .textSelection(.enabled)
            }
            IdentifierRow(label: Localized.Repo.changeColon) {
                IdentifierText(
                    value: detail.info.changeId.id,
                    prefixLength: Int(detail.info.changeId.shortLen),
                    prefixStyle: .changeId
                )
                CopyIconButton(value: detail.info.changeId.id, help: Localized.Repo.copyChange)
            }
            IdentifierRow(label: Localized.Repo.commitColon) {
                IdentifierText(
                    value: detail.info.commitId.id,
                    prefixLength: Int(detail.info.commitId.shortLen),
                    prefixStyle: .commitId
                )
                .accessibilityIdentifier(AID.Detail.commitId)
                .accessibilityValue(detail.info.commitId.id)
                CopyIconButton(value: detail.info.commitId.id, help: Localized.Repo.copyCommit)
            }
            if !detail.info.parents.isEmpty {
                IdentifierRow(label: Localized.Repo.parentsColon) {
                    IdentifierText(value: detail.info.parents.map { String($0.prefix(12)) }.joined(separator: ", "))
                }
            }
            if !detail.info.bookmarks.isEmpty {
                IdentifierRow(label: Localized.Repo.bookmarksColon) {
                    ForEach(detail.info.bookmarks, id: \.self) { bookmarkChip($0, showsCopy: true) }
                }
            }
            if !detail.info.tags.isEmpty {
                IdentifierRow(label: Localized.Repo.tagsColon) {
                    ForEach(detail.info.tags, id: \.self) { tagChip($0) }
                }
            }
            if let visibleDiffStats {
                IdentifierRow(label: Localized.Repo.changesColon) { diffStatsView(visibleDiffStats, spelledOut: true) }
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
            bylineAuthors
            bylineSeparator
            IdentifierText(
                value: detail.info.changeId.compact,
                prefixLength: Int(detail.info.changeId.shortLen),
                prefixStyle: .changeId
            )
            CopyIconButton(value: detail.info.changeId.id, help: Localized.Repo.copyChange)
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

    private func person(_ person: CommitAuthor) -> some View {
        HStack(spacing: 5) {
            CommitAvatar(email: person.email, size: 14)
            Text(person.name)
                .jayjayFont(11)
            Text("<\(person.email)>")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .lineLimit(1)
                .truncationMode(.middle)
        }
        .textSelection(.enabled)
        .help(person.identity)
    }

    private var bylineAuthors: some View {
        let people = [detail.info.author] + detail.info.coAuthors
        return HStack(spacing: 5) {
            HStack(spacing: -4) {
                ForEach(Array(people.prefix(3).enumerated()), id: \.offset) { index, person in
                    CommitAvatar(email: person.email, size: 14)
                        .background(Circle().fill(.background).padding(-1.5))
                        .zIndex(Double(-index))
                }
            }
            Text(changeCreditLine(change: detail.info))
                .jayjayFont(11)
        }
        .textSelection(.enabled)
        .help(people.map(\.identity).joined(separator: "\n"))
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
                    .accessibilityLabel(Localized.Repo.combinedSelection)
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
                .help(Localized.Repo.reverseCompareDirection)
                .accessibilityIdentifier(AID.Compare.reverseDirection)
            }
            Text(compareDisplay?.title ?? Localized.Repo.comparing)
                .jayjayFont(12, weight: .medium)
                .accessibilityIdentifier(AID.Compare.banner)
            compareLabel(compareDisplay?.from ?? String(compareFromId?.prefix(8) ?? ""))
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            compareLabel(compareDisplay?.to ?? String(detailRevision.prefix(8)))
            Spacer()
            Text(Localized.Repo.filesChangedCount(count: detail.diff.count))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                onClearCompare?()
            } label: {
                Image(systemName: "xmark.circle.fill")
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
            .help(Localized.Repo.exitCompareMode)
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
        .help(Localized.Repo.tagNamed(name: name))
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
                CopyIconButton(value: name, help: Localized.Repo.copyBookmarkName)
            }
        }
        .lineLimit(1)
        .help(
            conflicted
                ? Localized.Repo.conflictedBookmarkNamed(name: name)
                : Localized.Repo.bookmarkNamed(name: name)
        )
    }

    private func compareLabel(_ text: String) -> some View {
        Text(text)
            .jayjayFont(12, weight: .semibold, design: .monospaced)
            .lineLimit(1)
            .truncationMode(.middle)
    }

    func formatTimestamp(_ millis: Int64) -> String {
        Date(timeIntervalSince1970: Double(millis) / 1000.0).formatted(.dateTime.year().month().day().hour().minute().locale(Localized.locale))
    }

    func conflictBar(hunk: DiffHunk) -> some View {
        let path = hunk.path
        return HStack(spacing: 10) {
            Image(systemName: "exclamationmark.triangle.fill")
                .foregroundStyle(.red)
            Text(Localized.Repo.conflict)
                .jayjayFont(12, weight: .semibold)
            Spacer()
            Button(Localized.Repo.useOurs) {
                actions?.resolveUseOurs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useOurs(path))
            Button(Localized.Repo.useTheirs) {
                actions?.resolveUseTheirs(rev: detailRevision, path: path)
            }
            .buttonStyle(.bordered)
            .accessibilityIdentifier(AID.Conflict.useTheirs(path))
            resolutionButton(path: path, options: ConflictResolution.options(
                mergeEditor: repo.flatMap { $0.isRemote() ? nil : $0.mergeEditorName() },
                supportsConflictEditor: Self.canEnterConflictEditor(info: detail.info, hunk: hunk, isCompareMode: isCompareMode),
                editor: appSettings.externalEditor
            ))
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.red.opacity(0.08))
    }

    @ViewBuilder private func resolutionButton(path: String, options: [ConflictResolution]) -> some View {
        if let primary = options.first {
            let others = options.dropFirst()
            Group {
                if others.isEmpty {
                    Button(primary.title(editor: appSettings.externalEditor)) { resolve(primary, path: path) }
                } else {
                    Menu {
                        ForEach(Array(others), id: \.self) { option in
                            Button(option.title(editor: appSettings.externalEditor)) { resolve(option, path: path) }
                                .disabled(isPreparing(option))
                        }
                    } label: {
                        Text(primary.title(editor: appSettings.externalEditor))
                    } primaryAction: {
                        resolve(primary, path: path)
                    }
                    .menuStyle(.button)
                    .fixedSize()
                }
            }
            .buttonStyle(.bordered)
            .disabled(isPreparing(primary))
            .accessibilityIdentifier(primary.accessibilityIdentifier(path: path))
        }
    }

    private func isPreparing(_ resolution: ConflictResolution) -> Bool {
        resolution == .inJayJay && conflictEditorPreparation != nil
    }

    private func resolve(_ resolution: ConflictResolution, path: String) {
        switch resolution {
            case .mergeEditor:
                actions?.resolveWithMergeEditor(rev: detailRevision, path: path)
            case .inJayJay:
                prepareConflictEditor(path: path)
            case let .editorTool(tool):
                actions?.resolveInEditor(rev: detailRevision, path: path, tool: tool)
            case .openInEditor:
                appSettings.openInEditor(filePath: path, repoPath: repoPath)
        }
    }
}

private extension CommitAuthor {
    var identity: String {
        "\(name) <\(email)>"
    }
}
