import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffEditView: View {
    let detail: ChangeDetail
    let repo: JayJayRepo?
    let diffStore: DiffStore
    let actions: (any ChangeActions)?
    let onDone: () -> Void

    @State var loadedFiles: [String: DiffEditLoadedFile] = [:]
    @State var selectedChangedLinesByPath: [String: Set<Int>] = [:]
    @State var newChangeMessage = ""
    @State var showEmptySelectionAlert = false
    @State var selectsNewlyLoadedFiles = false
    @State var isSelectingAll = false
    @State var bulkSelectionTask: Task<Void, Never>?
    @Environment(AppSettings.self) var settings

    var detailRevision: String {
        detail.info.selectionRevision
    }

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
                            rev: detailRevision,
                            commitId: detail.info.commitId.id,
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
        .onDisappear {
            bulkSelectionTask?.cancel()
        }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Label("Diff Edit", systemImage: "slider.horizontal.3")
                .jayjayFont(15, weight: .semibold)
            Text(String(detailRevision.prefix(12)))
                .jayjayFont(12, design: .monospaced)
                .foregroundStyle(.secondary)
            Spacer()
            Text(selectionSummary)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button {
                toggleBulkSelection()
            } label: {
                Label(selectionToggleTitle, systemImage: selectionToggleSystemImage)
            }
            .disabled(selectionToggleDisabled)
            .controlSize(.small)
            Button("Cancel", action: onDone)
        }
        .controlSize(.small)
        .padding(.horizontal, 18)
        .padding(.vertical, 12)
        .background(.background)
    }

    private var unsupportedNotice: some View {
        HStack(spacing: 10) {
            Image(systemName: "info.circle")
                .foregroundStyle(.secondary)
            Text("Projected, renamed, and non-text files can be previewed here but are not editable yet.")
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
            hunk.projection != nil
                || hunk.hunkType == .renamed
                || !DiffPlaceholder.isEditableText(hunk.oldContent)
                || !DiffPlaceholder.isEditableText(hunk.newContent)
        }
    }
}
