import JayJayCore
import SwiftUI

struct DiffEditView: View {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let actions: (any ChangeActions)?
    let onDone: () -> Void

    @State private var loadedFiles: [String: DiffEditLoadedFile] = [:]
    @State private var selectedChangedLinesByPath: [String: Set<Int>] = [:]
    @State private var newChangeMessage = ""
    @State private var showEmptySelectionAlert = false
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
                            selectedChangedLines: selectedChangedLinesByPath[hunk.path] ?? [],
                            onToggleFile: { toggleFileSelection(path: hunk.path) },
                            onSelectFile: { selectFile(path: hunk.path) },
                            onToggleLine: { toggleLineSelection(path: hunk.path, lineNumber: $0) },
                            onSelectHunk: { selectHunk(path: hunk.path, range: $0) },
                            onLoaded: { loaded in
                                loadedFiles[hunk.path] = loaded
                                syncSelection(path: hunk.path, loaded: loaded)
                            }
                        )
                    }
                }
                .padding(18)
            }
        }
        .safeAreaInset(edge: .bottom) {
            actionBar
        }
        .alert("Nothing Selected", isPresented: $showEmptySelectionAlert) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("Select at least one file, hunk, or line before applying diff edit.")
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
            Button("Cancel", action: onDone)
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
                Button("Done") {
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
        detail.diff.contains { hunk in
            hunk.hunkType == .renamed
                || !DiffPlaceholder.isEditableText(hunk.oldContent)
                || !DiffPlaceholder.isEditableText(hunk.newContent)
        }
    }

    private var selectionSummary: String {
        let selectedFiles = builtSelections().count
        let selectedLines = selectedChangedLinesByPath.reduce(into: 0) { count, entry in
            guard let loaded = loadedFiles[entry.key] else { return }
            count += loaded.changedLineCount(selectedLines: entry.value)
        }
        if selectedFiles == 0 {
            return "Select files, hunks, or line ranges to edit"
        }
        let fileLabel = selectedFiles == 1 ? "file" : "files"
        let lineLabel = selectedLines == 1 ? "line" : "lines"
        return "\(selectedFiles) \(fileLabel), \(selectedLines) \(lineLabel) selected"
    }

    private var hasVisibleSelection: Bool {
        selectedChangedLinesByPath.values.contains { !$0.isEmpty }
    }

    private func builtSelections(for destination: DiffEditDestination = .newChild) -> [DiffEditFileSelection] {
        loadedFiles.compactMap { path, loaded in
            let selectedLines = selectedChangedLinesByPath[path] ?? []
            if destination == .removeFromSource {
                return loaded.makeInverseSelection(selectedLines: selectedLines)
            }
            return loaded.makeSelection(selectedLines: selectedLines)
        }
    }

    private func apply(_ destination: DiffEditDestination) {
        guard hasVisibleSelection else {
            showEmptySelectionAlert = true
            return
        }
        let selections = builtSelections(for: destination)
        guard !selections.isEmpty else {
            showEmptySelectionAlert = true
            return
        }
        actions?.applyDiffSelection(
            rev: detail.info.changeId,
            destination: destination,
            selections: selections,
            message: newChangeMessage,
            ignoreWhitespace: settings.ignoreWhitespace
        )
        onDone()
    }

    private func syncSelection(path: String, loaded: DiffEditLoadedFile) {
        let changedLines = loaded.changedLineSet
        guard !changedLines.isEmpty else {
            selectedChangedLinesByPath.removeValue(forKey: path)
            return
        }

        if let existing = selectedChangedLinesByPath[path] {
            selectedChangedLinesByPath[path] = existing.intersection(changedLines)
        } else {
            selectedChangedLinesByPath[path] = []
        }
    }

    private func toggleFileSelection(path: String) {
        guard let loaded = loadedFiles[path] else { return }
        let changedLines = loaded.changedLineSet
        let selected = selectedChangedLinesByPath[path] ?? []
        if changedLines.isSubset(of: selected) {
            selectedChangedLinesByPath[path] = []
        } else {
            selectedChangedLinesByPath[path] = changedLines
        }
    }

    private func selectFile(path: String) {
        guard let loaded = loadedFiles[path] else { return }
        selectedChangedLinesByPath[path] = loaded.changedLineSet
    }

    private func toggleLineSelection(path: String, lineNumber: Int) {
        guard let loaded = loadedFiles[path], loaded.changedLineSet.contains(lineNumber) else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        if selected.contains(lineNumber) {
            selected.remove(lineNumber)
        } else {
            selected.insert(lineNumber)
        }
        selectedChangedLinesByPath[path] = selected
    }

    private func selectHunk(path: String, range: ClosedRange<Int>) {
        guard let loaded = loadedFiles[path] else { return }
        let changedLines = Set(loaded.changedLineNumbers.filter(range.contains))
        guard !changedLines.isEmpty else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        selected.formUnion(changedLines)
        selectedChangedLinesByPath[path] = selected
    }
}
