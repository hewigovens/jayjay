import AppKit
import JayJayCore
import SwiftUI

struct EvologView: View {
    @State private var viewModel: EvologViewModel
    @Environment(\.colorScheme) private var colorScheme
    @Environment(\.jayjayTypography) private var typography
    @Environment(AppSettings.self) private var appSettings
    @State private var entryListWidth = PaneLayout.secondaryPaneDefault
    @State private var fileListWidth = PaneLayout.secondaryPaneDefault
    let target: ChangeInfo?
    let isRestoring: Bool
    let onRestore: (String) -> Void
    let onDismiss: () -> Void

    init(
        entries: [EvologEntry],
        changeId: String,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        target: ChangeInfo?,
        isRestoring: Bool,
        onRestore: @escaping (String) -> Void,
        onDismiss: @escaping () -> Void
    ) {
        _viewModel = State(wrappedValue: EvologViewModel(
            entries: entries, changeId: changeId, repo: repo, diffStore: diffStore
        ))
        self.target = target
        self.isRestoring = isRestoring
        self.onRestore = onRestore
        self.onDismiss = onDismiss
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if viewModel.entries.isEmpty {
                ContentUnavailableView(
                    Localized.Repo.noEvolution,
                    systemImage: "clock.arrow.circlepath",
                    description: Text(Localized.Repo.thisChangeHasNoRecordedRewrites)
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ResizableSplit(
                    width: $entryListWidth,
                    range: PaneLayout.evologEntryListRange(detailWidth:),
                    onEnded: { appSettings.secondaryPaneWidth = $0 },
                    dividerIdentifier: AID.Evolog.entryListDivider,
                    leading: {
                        entryList
                            .accessibilityElement(children: .contain)
                            .accessibilityIdentifier(AID.Evolog.entryList)
                    },
                    trailing: {
                        diffPane
                    }
                )
            }
        }
        .onAppear {
            entryListWidth = appSettings.secondaryPaneWidth
            fileListWidth = appSettings.secondaryPaneWidth
        }
    }

    /// "Evolution: <change-id>" with the id's prefix highlighted (all entries share
    /// the change-id). Built as one AttributedString to avoid the deprecated `Text(+)`.
    private var headerLabel: Text {
        var label = AttributedString("\(Localized.Repo.evolutionPrefix) ")
        if let first = viewModel.entries.first {
            label.append(first.changeId.highlighted(
                scheme: colorScheme,
                font: typography.font(13, weight: .semibold, design: .monospaced)
            ))
        } else {
            label.append(AttributedString(String(viewModel.changeId.prefix(8))))
        }
        return Text(label)
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: "clock.arrow.circlepath")
                .foregroundStyle(.secondary)
            headerLabel
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .lineLimit(1)
            Spacer()
            Toggle(Localized.Repo.hideSnapshots, isOn: Binding(
                get: { viewModel.hideSnapshots },
                set: { viewModel.setHideSnapshots($0) }
            ))
            .toggleStyle(.checkbox)
            .jayjayFont(11)
            .help(Localized.Repo.hideConsecutiveWorkingCopySnapshots)
            .accessibilityIdentifier(AID.Evolog.hideSnapshots)
            Text(Localized.Evolog.versionCount(count: viewModel.entries.count))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button(Localized.Repo.done, action: onDismiss)
                .keyboardShortcut(.cancelAction)
                .help(Localized.Repo.closeEvolutionViewEsc)
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }

    private var entryList: some View {
        List {
            ForEach(viewModel.displayedRows, id: \.self) { row in
                Group {
                    if row.isCollapsedRun {
                        collapsedRunRow(row)
                    } else {
                        entryRow(entry: viewModel.entries[row.actionIndex])
                    }
                }
                .accessibilityElement(children: .combine)
                .accessibilityIdentifier(
                    row.isCollapsedRun
                        ? AID.Evolog.snapshotRun(start: row.actionIndex, count: Int(row.count))
                        : AID.Evolog.version(row.actionIndex)
                )
                .listRowBackground(
                    viewModel.isSelected(row)
                        ? Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10)
                        : Color.clear
                )
                .accessibilityAddTraits(
                    viewModel.isSelected(row) ? .isSelected : []
                )
                .onTapGesture {
                    viewModel.select(
                        row,
                        click: SelectionClick(modifiers: NSEvent.modifierFlags)
                    )
                }
            }
        }
        .listStyle(.plain)
    }

    private func collapsedRunRow(_ row: EvologRow) -> some View {
        let newest = viewModel.entries[row.actionIndex]
        return HStack(spacing: 6) {
            Image(systemName: "chevron.right")
                .jayjayFont(10)
                .foregroundStyle(.tertiary)
            Image(systemName: "camera")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            Text(Localized.Evolog.snapshotCount(count: Int(row.count)))
                .jayjayFont(12, weight: .medium)
                .lineLimit(1)
            Spacer()
            Text(EvologDisplay.timestamp(newest.timestampMillis))
                .jayjayFont(10)
                .foregroundStyle(.tertiary)
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
        .contextMenu {
            copyMenu(commitId: newest.commitId.id)
        }
    }

    private func entryRow(entry: EvologEntry) -> some View {
        VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 6) {
                Image(systemName: EvologDisplay.operationIcon(entry.operation))
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
                Text(EvologDisplay.operationLabel(entry.operation))
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
                Spacer()
                Text(EvologDisplay.timestamp(entry.timestampMillis))
                    .jayjayFont(10)
                    .foregroundStyle(.tertiary)
            }
            Text(EvologDisplay.descriptionLabel(entry.description))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .lineLimit(1)
            Text(entry.commitId.highlighted(
                scheme: colorScheme,
                font: typography.font(10, design: .monospaced)
            ))
            .lineLimit(1)
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
        .contextMenu {
            copyMenu(commitId: entry.commitId.id)
        }
    }

    @ViewBuilder
    private func copyMenu(commitId: String) -> some View {
        Button {
            onRestore(commitId)
        } label: {
            Label(Localized.Repo.restoreThisVersion, systemImage: "arrow.uturn.backward")
        }
        .disabled(isRestoring || target?.isImmutable == true)
        .help(restoreHelp)
        Button {
            viewModel.copyCommitId(commitId)
        } label: {
            Label(Localized.Menu.copyCommitId, systemImage: "doc.on.doc")
        }
        Button {
            viewModel.copyRestoreCommand(commitId, into: restoreIntoRev)
        } label: {
            Label(Localized.Repo.copyJjRestoreCommand, systemImage: "terminal")
        }
    }

    private var restoreIntoRev: String {
        if target?.isWorkingCopy == true {
            return "@"
        }
        return target?.selectionRevision ?? viewModel.changeId
    }

    private var restoreHelp: String {
        if target?.isImmutable == true {
            return Localized.Repo.immutableChangesCannotBeRestored
        }
        return Localized.Repo.restoreVersionInto(rev: restoreIntoRev)
    }

    private var diffPane: some View {
        VStack(spacing: 0) {
            if let from = viewModel.selectedFromCommitId,
               let to = viewModel.selectedToCommitId
            {
                comparisonHeader(from: from, to: to, fileCount: viewModel.interdiffDetail?.diff.count)
                Divider()
            }
            diffContent
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
    }

    @ViewBuilder
    private var diffContent: some View {
        if viewModel.selectedIndex == nil {
            ContentUnavailableView(
                Localized.Repo.selectAVersion,
                systemImage: "arrow.left",
                description: Text(Localized.Repo.clickARowOnTheLeft)
            )
        } else if viewModel.interdiffLoading {
            ProgressView().controlSize(.small)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let error = viewModel.interdiffError {
            loadError(error) { viewModel.retryInterdiff() }
        } else if let detail = viewModel.interdiffDetail {
            if detail.diff.isEmpty {
                ContentUnavailableView(
                    Localized.Repo.noFileChanges,
                    systemImage: "equal.circle",
                    description: Text(Localized.Repo.descriptionOnlyOrIdenticalToThe)
                )
            } else {
                interdiffContent(detail: detail)
            }
        }
    }

    private func comparisonHeader(from: String, to: String, fileCount: Int?) -> some View {
        HStack(spacing: 8) {
            Button {
                viewModel.reverseComparison()
            } label: {
                Image(systemName: "arrow.left.arrow.right")
                    .foregroundStyle(.orange)
            }
            .buttonStyle(.plain)
            .disabled(!viewModel.canReverseComparison)
            .help(Localized.Repo.reverseComparisonDirection)
            .accessibilityIdentifier(AID.Evolog.reverseComparison)
            Text(Localized.Repo.comparingVersions)
                .jayjayFont(12, weight: .medium)
                .accessibilityIdentifier(AID.Evolog.comparisonBanner)
            versionLabel(from)
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            versionLabel(to)
            Spacer()
            if let fileCount {
                Text(Localized.Repo.filesChangedCount(count: fileCount))
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(.orange.opacity(0.08))
    }

    private func versionLabel(_ commitId: String) -> some View {
        Text(String(commitId.prefix(12)))
            .jayjayFont(12, weight: .semibold, design: .monospaced)
            .lineLimit(1)
    }

    private func interdiffContent(detail: ChangeDetail) -> some View {
        ResizableSplit(
            width: $fileListWidth,
            range: PaneLayout.evologFileListRange(paneWidth:),
            onEnded: { _ in },
            dividerIdentifier: AID.Evolog.fileListDivider,
            leading: {
                fileList(detail: detail)
                    .accessibilityElement(children: .contain)
                    .accessibilityIdentifier(AID.Evolog.fileList)
            },
            trailing: {
                if viewModel.fileLoading {
                    ProgressView().controlSize(.small)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else if let error = viewModel.fileError {
                    loadError(error) { viewModel.loadFile(path: viewModel.selectedPath) }
                } else if let hunk = viewModel.selectedHunk,
                          let from = viewModel.selectedFromCommitId,
                          let to = viewModel.selectedToCommitId
                {
                    DiffSection(
                        hunk: hunk,
                        rev: to,
                        repo: viewModel.repo,
                        actions: nil,
                        isWorkingCopy: false,
                        diffStore: viewModel.diffStore,
                        reviewStore: nil,
                        noteEditor: .constant(nil),
                        compareFromRev: from
                    )
                    .id("\(from)|\(hunk.path)")
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ContentUnavailableView(
                        Localized.Repo.selectAFile,
                        systemImage: "doc",
                        description: Text(Localized.Repo.pickAFileFromTheList)
                    )
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        )
    }

    private func loadError(_ message: String, retry: @escaping () -> Void) -> some View {
        ContentUnavailableView {
            Label(Localized.Repo.couldntLoadDiff, systemImage: "exclamationmark.triangle")
        } description: {
            Text(message)
        } actions: {
            Button(Localized.Repo.retry, action: retry)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func fileList(detail: ChangeDetail) -> some View {
        List(
            detail.diff,
            id: \.path,
            selection: Binding(
                get: { viewModel.selectedPath },
                set: { viewModel.selectedPath = $0 }
            )
        ) { hunk in
            HStack(spacing: 6) {
                Image(systemName: EvologDisplay.hunkIcon(hunk.hunkType))
                    .jayjayFont(11)
                    .foregroundStyle(EvologDisplay.hunkColor(hunk.hunkType))
                Text(hunk.path)
                    .jayjayFont(11, design: .monospaced)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            .tag(hunk.path)
        }
        .listStyle(.plain)
        .onChange(of: viewModel.selectedPath) { _, newPath in
            viewModel.loadFile(path: newPath)
        }
    }
}
