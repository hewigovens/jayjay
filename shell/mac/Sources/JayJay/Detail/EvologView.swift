import JayJayCore
import SwiftUI

struct EvologView: View {
    @State private var viewModel: EvologViewModel
    @Environment(AppSettings.self) private var settings
    let onDismiss: () -> Void
    let onRestoreCommit: (String) -> Void

    init(
        entries: [EvologEntry],
        changeId: String,
        repo: JayJayRepo?,
        diffStore: DiffStore,
        onDismiss: @escaping () -> Void,
        onRestoreCommit: @escaping (String) -> Void
    ) {
        _viewModel = State(wrappedValue: EvologViewModel(
            entries: entries, changeId: changeId, repo: repo, diffStore: diffStore
        ))
        self.onDismiss = onDismiss
        self.onRestoreCommit = onRestoreCommit
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
                if viewModel.visibleRows.isEmpty {
                    ContentUnavailableView(
                        "Snapshots Hidden",
                        systemImage: "camera",
                        description: Text("Turn off Hide snapshots to see these versions.")
                    )
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                } else {
                    HSplitView {
                        entryList
                            .frame(minWidth: 260, idealWidth: 300, maxWidth: 380)
                        diffPane
                            .frame(maxWidth: .infinity, maxHeight: .infinity)
                    }
                }
            }
        }
    }

    private var header: some View {
        HStack(spacing: 10) {
            Image(systemName: "clock.arrow.circlepath")
                .foregroundStyle(.secondary)
            Text("Evolution: \(String(viewModel.changeId.prefix(8)))")
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .lineLimit(1)
            Spacer()
            Toggle("Hide snapshots", isOn: Binding(
                get: { viewModel.hideSnapshots },
                set: {
                    settings.evologHideSnapshots = $0
                    viewModel.setHideSnapshots($0)
                }
            ))
            .toggleStyle(.checkbox)
            .jayjayFont(11)
            Toggle("Collapse runs", isOn: Binding(
                get: { viewModel.collapseSnapshotRuns },
                set: {
                    settings.evologCollapseSnapshotRuns = $0
                    viewModel.setCollapseSnapshotRuns($0)
                }
            ))
            .toggleStyle(.checkbox)
            .jayjayFont(11)
            .disabled(viewModel.hideSnapshots)
            Text(versionSummary)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button("Done", action: onDismiss)
                .keyboardShortcut(.cancelAction)
                .help("Close evolution view (esc)")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .onAppear {
            viewModel.setHideSnapshots(settings.evologHideSnapshots)
            viewModel.setCollapseSnapshotRuns(settings.evologCollapseSnapshotRuns)
        }
    }

    private var entryList: some View {
        List(
            viewModel.visibleRows,
            id: \.id,
            selection: Binding(
                get: { viewModel.selectedIndex },
                set: { viewModel.selectedIndex = $0 }
            )
        ) { row in
            entryRow(row: row).tag(Int(row.primaryIndex))
        }
        .listStyle(.plain)
        .onChange(of: viewModel.selectedIndex) { _, newIndex in
            viewModel.loadInterdiff(for: newIndex)
        }
    }

    private var versionSummary: String {
        let visible = viewModel.visibleRows.count
        let suffix = visible == 1 ? "" : "s"
        if viewModel.hiddenSnapshotCount > 0 {
            return "\(visible) version\(suffix), \(viewModel.hiddenSnapshotCount) hidden"
        }
        return "\(visible) version\(suffix)"
    }

    private func entryRow(row: EvologVisibleRow) -> some View {
        let entry = row.primary
        return VStack(alignment: .leading, spacing: 3) {
            HStack(spacing: 6) {
                Image(systemName: row.isSnapshotRun ? "camera.on.rectangle" : EvologDisplay
                    .operationIcon(entry.operation))
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
                Text(row.isSnapshotRun ? "\(row.entries.count) snapshots" : EvologDisplay
                    .operationLabel(entry.operation))
                    .jayjayFont(12, weight: .medium)
                    .lineLimit(1)
                Spacer()
                Text(EvologDisplay.timestamp(entry.timestampMillis))
                    .jayjayFont(10)
                    .foregroundStyle(.tertiary)
            }
            HStack(spacing: 6) {
                Text(String(entry.commitId.prefix(12)))
                    .jayjayFont(10, design: .monospaced)
                    .foregroundStyle(Color.accentColor.opacity(0.8))
                if let subtitle = rowSubtitle(row) {
                    Text(subtitle)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
            restoreButton(commitId: entry.commitId)
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
        .contextMenu {
            Button {
                viewModel.copyCommitId(entry.commitId)
            } label: {
                Label("Copy Commit ID", systemImage: "doc.on.doc")
            }
            Button {
                viewModel.copyRestoreCommand(entry.commitId)
            } label: {
                Label("Copy ‘jj restore’ command", systemImage: "terminal")
            }
            Button {
                onRestoreCommit(entry.commitId)
            } label: {
                Label("Restore to @", systemImage: "arrow.counterclockwise")
            }
            .disabled(entry.commitId == viewModel.headCommitId)
        }
    }

    private func rowSubtitle(_ row: EvologVisibleRow) -> String? {
        if row.isSnapshotRun, let last = row.entries.last {
            return "\(String(row.primary.commitId.prefix(8)))...\(String(last.commitId.prefix(8)))"
        }
        return row.primary.description.isEmpty ? nil : row.primary.description
    }

    private func restoreButton(commitId: String) -> some View {
        Button {
            onRestoreCommit(commitId)
        } label: {
            Label("Restore to @", systemImage: "arrow.counterclockwise")
                .jayjayFont(10, weight: .medium)
        }
        .buttonStyle(.borderless)
        .controlSize(.small)
        .disabled(commitId == viewModel.headCommitId)
    }

    @ViewBuilder
    private var diffPane: some View {
        if viewModel.selectedIndex == nil {
            ContentUnavailableView(
                "Select a Version",
                systemImage: "arrow.left",
                description: Text("Click a row on the left to see what changed since that version.")
            )
        } else if viewModel.interdiffLoading {
            ProgressView().controlSize(.small)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
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

    private func interdiffContent(detail: ChangeDetail) -> some View {
        HSplitView {
            fileList(detail: detail)
                .frame(minWidth: 180, idealWidth: 220, maxWidth: 320)
            if let hunk = viewModel.selectedHunk,
               let from = viewModel.selectedFromCommitId,
               let to = viewModel.headCommitId
            {
                DiffSection(
                    hunk: hunk,
                    rev: to,
                    repo: viewModel.repo,
                    actions: nil,
                    isWorkingCopy: false,
                    diffStore: viewModel.diffStore,
                    reviewStore: nil,
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
