import JayJayCore
import SwiftUI

struct DiffEditView: View {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let actions: (any ChangeActions)?
    let onDone: () -> Void

    @State private var selectionModes: [String: DiffEditSelectionMode] = [:]
    @State private var loadedFiles: [String: DiffEditLoadedFile] = [:]
    @State private var newChangeMessage = ""
    @Environment(AppSettings.self) private var settings

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 14) {
                    if hasUnsupportedFiles {
                        unsupportedNotice
                    }
                    ForEach(detail.diff, id: \.path) { hunk in
                        DiffEditFileSection(
                            hunk: hunk,
                            rev: detail.info.changeId,
                            repo: repo,
                            selectionMode: binding(for: hunk.path),
                            onLoaded: { loadedFiles[hunk.path] = $0 }
                        )
                    }
                }
                .padding(18)
            }
        }
        .safeAreaInset(edge: .bottom) {
            actionBar
        }
        .onAppear {
            newChangeMessage = detail.info.description
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Label("Diff Edit", systemImage: "slider.horizontal.3")
                .jayjayFont(15, weight: .semibold)
            Text(String(detail.info.changeId.prefix(12)))
                .jayjayFont(12, design: .monospaced)
                .foregroundStyle(.secondary)
            Spacer()
            Text(selectionSummary)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button("Done", action: onDone)
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .background(.background)
    }

    private var unsupportedNotice: some View {
        HStack(spacing: 10) {
            Image(systemName: "info.circle")
                .foregroundStyle(.secondary)
            Text("Renames and non-text files can be previewed here but are not editable yet.")
                .jayjayFont(12)
                .foregroundStyle(.secondary)
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var actionBar: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                Text(selectionSummary)
                    .jayjayFont(12, weight: .medium)
                Spacer()
                if !detail.info.isWorkingCopy {
                    TextField("New change description", text: $newChangeMessage)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 260)
                }
                if !detail.info.isWorkingCopy {
                    Button("Create New Child Change") { apply(.newChild) }
                        .buttonStyle(.borderedProminent)
                    Button("Create Parallel Change") { apply(.newParallel) }
                        .buttonStyle(.bordered)
                    Button("Move to Working Copy") { apply(.moveToWorkingCopy) }
                        .buttonStyle(.bordered)
                }
                Button(detail.info.isWorkingCopy ? "Discard Selected Changes" : "Remove Selected Changes") {
                    apply(.removeFromSource)
                }
                .buttonStyle(.bordered)
            }
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
    }

    private var hasUnsupportedFiles: Bool {
        detail.diff.contains { $0.hunkType == .renamed }
    }

    private var selectionSummary: String {
        let selectedFiles = builtSelections().count
        let selectedLines = selectionModes.reduce(into: 0) { count, entry in
            guard let loaded = loadedFiles[entry.key] else { return }
            count += loaded.changedLineCount(for: entry.value)
        }
        if selectedFiles == 0 {
            return "Select files, hunks, or line ranges to edit"
        }
        let fileLabel = selectedFiles == 1 ? "file" : "files"
        let lineLabel = selectedLines == 1 ? "line" : "lines"
        return "\(selectedFiles) \(fileLabel), \(selectedLines) \(lineLabel) selected"
    }

    private func binding(for path: String) -> Binding<DiffEditSelectionMode?> {
        Binding(
            get: { selectionModes[path] },
            set: {
                if let value = $0 {
                    selectionModes[path] = value
                } else {
                    selectionModes.removeValue(forKey: path)
                }
            }
        )
    }

    private func builtSelections() -> [DiffEditFileSelection] {
        selectionModes.compactMap { path, mode in
            loadedFiles[path]?.makeSelection(mode: mode)
        }
    }

    private func apply(_ destination: DiffEditDestination) {
        let selections = builtSelections()
        guard !selections.isEmpty else { return }
        actions?.applyDiffSelection(
            rev: detail.info.changeId,
            destination: destination,
            selections: selections,
            message: newChangeMessage,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        onDone()
    }
}
