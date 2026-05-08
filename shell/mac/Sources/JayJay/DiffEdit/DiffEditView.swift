import JayJayCore
import SwiftUI

struct DiffEditView: View {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let actions: (any ChangeActions)?
    let onDone: () -> Void

    @State private var loadedFiles: [String: DiffEditLoadedFile] = [:]
    @State private var selectedChangedLinesByPath: [String: Set<Int>] = [:]
    @State private var newChangeMessage = ""
    @State private var showEmptySelectionAlert = false
    @State private var selectAllAsFilesLoad = false
    @State private var isPreparingSelectAll = false
    @State private var selectAllLoadTask: Task<Void, Never>?
    @State private var selectAllGeneration = 0
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
                            diffStore: diffStore,
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
            Button("Select All") { selectAllChanges() }
                .controlSize(.small)
                .disabled(isPreparingSelectAll)
            Button("Clear") { clearSelection() }
                .controlSize(.small)
                .disabled(!hasVisibleSelection && !isPreparingSelectAll)
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
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 10) {
                Image(systemName: "info.circle")
                    .foregroundStyle(.secondary)
                Text(unsupportedSummary)
                    .jayjayFont(12, weight: .medium)
                    .foregroundStyle(.secondary)
                Spacer()
            }
            ForEach(unsupportedDetails.prefix(4), id: \.self) { detail in
                Text(detail)
                    .jayjayFont(11, design: .monospaced)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.middle)
            }
            if unsupportedDetails.count > 4 {
                Text("\(unsupportedDetails.count - 4) more unsupported file\(unsupportedDetails.count == 5 ? "" : "s")")
                    .jayjayFont(11)
                    .foregroundStyle(.tertiary)
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }

    private var actionBar: some View {
        VStack(spacing: 10) {
            Divider()
            HStack(spacing: 12) {
                VStack(alignment: .leading, spacing: 3) {
                    Text(selectionSummary)
                        .jayjayFont(12, weight: .medium)
                    Text(topologyHelp)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                }
                Spacer()
                if !detail.info.isWorkingCopy {
                    TextField("New change description", text: $newChangeMessage)
                        .textFieldStyle(.roundedBorder)
                        .frame(width: 260)
                }
                if !detail.info.isWorkingCopy {
                    Button("Create New Child Change") { apply(.newChild) }
                        .buttonStyle(.borderedProminent)
                        .disabled(isPreparingSelectAll)
                    Button("Create Parallel Change") { apply(.newParallel) }
                        .buttonStyle(.bordered)
                        .disabled(isPreparingSelectAll)
                    Button("Move to Working Copy") { apply(.moveToWorkingCopy) }
                        .buttonStyle(.bordered)
                        .disabled(isPreparingSelectAll)
                }
                Button("Done") {
                    apply(.removeFromSource)
                }
                .buttonStyle(.bordered)
                .disabled(isPreparingSelectAll)
            }
            .padding(.horizontal, 18)
            .padding(.bottom, 12)
        }
        .background(.bar)
    }

    private var hasUnsupportedFiles: Bool {
        !unsupportedDetails.isEmpty
    }

    private var unsupportedDetails: [String] {
        detail.diff.compactMap { hunk in
            let loaded = loadedFiles[hunk.path]
            return diffEditUnsupportedReason(
                hunkType: hunk.hunkType,
                oldContent: loaded?.oldContent ?? hunk.oldContent,
                newContent: loaded?.newContent ?? hunk.newContent
            ).map { "\(hunk.path) — \($0)" }
        }
    }

    private var unsupportedSummary: String {
        "\(unsupportedDetails.count) file\(unsupportedDetails.count == 1 ? "" : "s") can be previewed but not edited"
    }

    private var topologyHelp: String {
        if detail.info.isWorkingCopy {
            return "Done keeps selected changes in @ and abandons unchecked changes."
        }
        return "Child/Parallel/Working Copy extract selected changes; Done keeps selected changes here and removes unchecked changes."
    }

    private var selectionSummary: String {
        if isPreparingSelectAll {
            return "Loading all editable files..."
        }
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
        buildDiffEditSelections(
            loadedFiles: loadedFiles,
            selectedChangedLinesByPath: selectedChangedLinesByPath,
            destination: destination
        )
    }

    private func apply(_ destination: DiffEditDestination) {
        Task { await applyPrepared(destination) }
    }

    @MainActor
    private func applyPrepared(_ destination: DiffEditDestination) async {
        if selectAllAsFilesLoad, let selectAllLoadTask {
            await selectAllLoadTask.value
        }
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
        } else if selectAllAsFilesLoad, loaded.supportsDiffEdit {
            selectedChangedLinesByPath[path] = changedLines
        } else {
            selectedChangedLinesByPath[path] = []
        }
    }

    private func toggleFileSelection(path: String) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        let changedLines = loaded.changedLineSet
        let selected = selectedChangedLinesByPath[path] ?? []
        if changedLines.isSubset(of: selected) {
            selectedChangedLinesByPath[path] = []
        } else {
            selectedChangedLinesByPath[path] = changedLines
        }
    }

    private func selectFile(path: String) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        selectedChangedLinesByPath[path] = loaded.changedLineSet
    }

    private func toggleLineSelection(path: String, lineNumber: Int) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit,
              loaded.changedLineSet.contains(lineNumber) else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        if selected.contains(lineNumber) {
            selected.remove(lineNumber)
        } else {
            selected.insert(lineNumber)
        }
        selectedChangedLinesByPath[path] = selected
    }

    private func selectHunk(path: String, range: ClosedRange<Int>) {
        guard let loaded = loadedFiles[path], loaded.supportsDiffEdit else { return }
        let changedLines = Set(loaded.changedLineNumbers.filter(range.contains))
        guard !changedLines.isEmpty else { return }
        var selected = selectedChangedLinesByPath[path] ?? []
        selected.formUnion(changedLines)
        selectedChangedLinesByPath[path] = selected
    }

    private func selectAllChanges() {
        selectAllGeneration += 1
        selectAllAsFilesLoad = true
        isPreparingSelectAll = true
        for (path, loaded) in loadedFiles where loaded.supportsDiffEdit {
            selectedChangedLinesByPath[path] = loaded.changedLineSet
        }
        let generation = selectAllGeneration
        let task = Task { @MainActor in
            await loadAllFilesForSelectAll(generation: generation)
        }
        selectAllLoadTask = task
    }

    private func clearSelection() {
        selectAllGeneration += 1
        selectAllAsFilesLoad = false
        isPreparingSelectAll = false
        selectAllLoadTask?.cancel()
        selectAllLoadTask = nil
        selectedChangedLinesByPath = loadedFiles.keys.reduce(into: [:]) { result, path in
            result[path] = []
        }
    }

    @MainActor
    private func loadAllFilesForSelectAll(generation: Int) async {
        defer {
            if generation == selectAllGeneration {
                isPreparingSelectAll = false
                selectAllLoadTask = nil
            }
        }

        for hunk in detail.diff {
            if Task.isCancelled { return }
            let loaded: DiffEditLoadedFile?
            if let existing = loadedFiles[hunk.path] {
                loaded = existing
            } else {
                loaded = await loadDiffEditFile(
                    hunk: hunk,
                    rev: detail.info.changeId,
                    repo: repo,
                    diffStore: diffStore,
                    ignoreWhitespace: settings.ignoreWhitespace
                )
            }

            guard generation == selectAllGeneration else { return }
            guard let loaded else { continue }
            loadedFiles[hunk.path] = loaded
            syncSelection(path: hunk.path, loaded: loaded)
            if loaded.supportsDiffEdit {
                selectedChangedLinesByPath[hunk.path] = loaded.changedLineSet
            }
        }
    }
}
