import AppKit
import JayJayCore
import SwiftUI

struct EvologView: View {
    @State private var viewModel: EvologViewModel
    @Environment(\.colorScheme) private var colorScheme
    @Environment(AppSettings.self) private var appSettings
    @State private var snapshotsWidth = PaneLayout.secondaryPaneWidthDefault
    @State private var filenamesWidth = PaneLayout.secondaryPaneWidthDefault
    let onDismiss: () -> Void

    init(
        entries: [EvologEntry],
        changeId: String,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        onDismiss: @escaping () -> Void
    ) {
        _viewModel = State(wrappedValue: EvologViewModel(
            entries: entries, changeId: changeId, repo: repo, diffStore: diffStore
        ))
        self.onDismiss = onDismiss
    }

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if viewModel.entries.isEmpty {
                ContentUnavailableView(
                    "No Evolution",
                    systemImage: "clock.arrow.circlepath",
                    description: Text("This change has no recorded rewrites.")
                )
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                GeometryReader { geo in
                    let range = PaneLayout.evologSnapshotsRange(contentWidth: geo.size.width)
                    let width = Binding(get: { min(snapshotsWidth, range.upperBound) }, set: { snapshotsWidth = $0 })
                    HStack(spacing: 0) {
                        entryList
                            .frame(width: width.wrappedValue)
                            .accessibilityElement(children: .contain)
                            .accessibilityIdentifier(AID.Evolog.entryList)
                        SidebarDivider(
                            position: width,
                            range: range,
                            onEnded: { appSettings.secondaryPaneWidth = $0 }
                        )
                        .accessibilityElement()
                        .accessibilityIdentifier(AID.Evolog.entryListDivider)
                        diffPane
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                }
            }
        }
        .onAppear {
            snapshotsWidth = appSettings.secondaryPaneWidth
            filenamesWidth = appSettings.secondaryPaneWidth
        }
    }

    /// "Evolution: <change-id>" with the id's prefix highlighted (all entries share
    /// the change-id). Built as one AttributedString to avoid the deprecated `Text(+)`.
    private var headerLabel: Text {
        var label = AttributedString("Evolution: ")
        if let first = viewModel.entries.first {
            label.append(first.changeId.highlighted(scheme: colorScheme, maxChars: 8))
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
            Toggle("Hide snapshots", isOn: Binding(
                get: { viewModel.hideSnapshots },
                set: { viewModel.setHideSnapshots($0) }
            ))
            .toggleStyle(.checkbox)
            .jayjayFont(11)
            .help("Hide consecutive working-copy snapshots")
            .accessibilityIdentifier(AID.Evolog.hideSnapshots)
            Text("\(viewModel.entries.count) version\(viewModel.entries.count == 1 ? "" : "s")")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button("Done", action: onDismiss)
                .keyboardShortcut(.cancelAction)
                .help("Close evolution view (esc)")
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
                    viewModel.selection.contains(row.actionIndex)
                        ? Color.accentColor.opacity(colorScheme == .dark ? 0.18 : 0.10)
                        : Color.clear
                )
                .accessibilityAddTraits(
                    viewModel.selection.contains(row.actionIndex) ? .isSelected : []
                )
                .onTapGesture {
                    viewModel.select(
                        row,
                        click: OrderedSelectionClick(modifiers: NSEvent.modifierFlags)
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
            Text("\(row.count) snapshots")
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
            Text(entry.commitId.highlighted(scheme: colorScheme))
                .jayjayFont(10, design: .monospaced)
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
            viewModel.copyCommitId(commitId)
        } label: {
            Label("Copy Commit ID", systemImage: "doc.on.doc")
        }
        Button {
            viewModel.copyRestoreCommand(commitId)
        } label: {
            Label("Copy ‘jj restore’ command", systemImage: "terminal")
        }
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
                "Select a Version",
                systemImage: "arrow.left",
                description: Text("Click a row on the left to see what changed since that version.")
            )
        } else if viewModel.interdiffLoading {
            ProgressView().controlSize(.small)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
        } else if let error = viewModel.interdiffError {
            loadError(error) { viewModel.retryInterdiff() }
        } else if let detail = viewModel.interdiffDetail {
            if detail.diff.isEmpty {
                ContentUnavailableView(
                    "No File Changes",
                    systemImage: "equal.circle",
                    description: Text("Description-only or identical to the current version.")
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
            .help("Reverse comparison direction")
            .accessibilityIdentifier(AID.Evolog.reverseComparison)
            Text("Comparing Versions")
                .jayjayFont(12, weight: .medium)
                .accessibilityIdentifier(AID.Evolog.comparisonBanner)
            versionLabel(from)
            Image(systemName: "arrow.right")
                .jayjayFont(10)
                .foregroundStyle(.secondary)
            versionLabel(to)
            Spacer()
            if let fileCount {
                Text(fileCount == 1 ? "1 file changed" : "\(fileCount) files changed")
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
        GeometryReader { geo in
            let range = PaneLayout.evologFilenamesRange(detailWidth: geo.size.width)
            let width = Binding(get: { min(filenamesWidth, range.upperBound) }, set: { filenamesWidth = $0 })
            HStack(spacing: 0) {
                fileList(detail: detail)
                    .frame(width: width.wrappedValue)
                    .accessibilityElement(children: .contain)
                    .accessibilityIdentifier(AID.Evolog.fileList)
                SidebarDivider(
                    position: width,
                    range: range,
                    onEnded: { appSettings.secondaryPaneWidth = $0 }
                )
                .accessibilityElement()
                .accessibilityIdentifier(AID.Evolog.fileListDivider)
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
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    ContentUnavailableView(
                        "Select a File",
                        systemImage: "doc",
                        description: Text("Pick a file from the list to see its diff.")
                    )
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
        }
    }

    private func loadError(_ message: String, retry: @escaping () -> Void) -> some View {
        ContentUnavailableView {
            Label("Couldn’t Load Diff", systemImage: "exclamationmark.triangle")
        } description: {
            Text(message)
        } actions: {
            Button("Retry", action: retry)
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
            .accessibilityIdentifier(AID.Evolog.file(hunk.path))
        }
        .listStyle(.plain)
        .onChange(of: viewModel.selectedPath) { _, newPath in
            viewModel.loadFile(path: newPath)
        }
    }
}
