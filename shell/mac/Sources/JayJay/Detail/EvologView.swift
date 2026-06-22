import JayJayCore
import SwiftUI

struct EvologView: View {
    @State private var viewModel: EvologViewModel
    @Environment(\.colorScheme) private var colorScheme
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
                HSplitView {
                    entryList
                        .frame(minWidth: 240, idealWidth: 280, maxWidth: 360)
                    diffPane
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                }
            }
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
        List(
            Array(viewModel.entries.enumerated()),
            id: \.offset,
            selection: Binding(
                get: { viewModel.selectedIndex },
                set: { viewModel.selectedIndex = $0 }
            )
        ) { idx, entry in
            entryRow(idx: idx, entry: entry).tag(idx)
        }
        .listStyle(.plain)
        .onChange(of: viewModel.selectedIndex) { _, newIndex in
            viewModel.loadInterdiff(for: newIndex)
        }
    }

    private func entryRow(idx _: Int, entry: EvologEntry) -> some View {
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
            HStack(spacing: 6) {
                Text(entry.commitId.highlighted(scheme: colorScheme))
                    .jayjayFont(10, design: .monospaced)
                    .lineLimit(1)
                if !entry.description.isEmpty {
                    Text(entry.description)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }
        }
        .padding(.vertical, 2)
        .contentShape(Rectangle())
        .contextMenu {
            Button {
                viewModel.copyCommitId(entry.commitId.id)
            } label: {
                Label("Copy Commit ID", systemImage: "doc.on.doc")
            }
            Button {
                viewModel.copyRestoreCommand(entry.commitId.id)
            } label: {
                Label("Copy ‘jj restore’ command", systemImage: "terminal")
            }
        }
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
