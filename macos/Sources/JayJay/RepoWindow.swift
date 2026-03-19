import SwiftUI
import JayJayBindings

struct RepoWindow: View {
    let repoPath: String
    @State private var viewModel: RepoViewModel?
    @State private var initError: String?

    var body: some View {
        Group {
            if let vm = viewModel {
                RepoContentView(viewModel: vm)
            } else if let err = initError {
                VStack(spacing: 12) {
                    Image(systemName: "exclamationmark.triangle")
                        .font(.largeTitle)
                        .foregroundStyle(.red)
                    Text("Failed to open repository")
                        .font(.headline)
                    Text(err)
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ProgressView("Loading repository...")
            }
        }
        .task {
            do {
                let vm = try RepoViewModel(path: repoPath)
                viewModel = vm
                vm.refresh()
            } catch {
                initError = error.localizedDescription
            }
        }
        .navigationTitle(URL(fileURLWithPath: repoPath).lastPathComponent)
    }
}

struct RepoContentView: View {
    @Bindable var viewModel: RepoViewModel
    @State private var revsetDraft = ""

    var body: some View {
        HSplitView {
            DAGView(
                changes: viewModel.changes,
                selectedId: Binding(
                    get: { viewModel.selectedChangeId },
                    set: { viewModel.select(changeId: $0) }
                )
            )
            .frame(minWidth: 300, idealWidth: 400)

            DetailView(
                detail: viewModel.selectedChange,
                onDescribe: { rev, msg in viewModel.describe(rev: rev, message: msg) }
            )
            .frame(minWidth: 300, idealWidth: 500)
        }
        .onAppear {
            revsetDraft = viewModel.revset
        }
        .toolbar {
            ToolbarItem(placement: .principal) {
                HStack(spacing: 8) {
                    TextField("Revset", text: $revsetDraft)
                        .textFieldStyle(.roundedBorder)
                        .font(.caption.monospaced())
                        .frame(minWidth: 260, idealWidth: 420)
                        .onSubmit {
                            applyRevset()
                        }

                    Button("Apply") {
                        applyRevset()
                    }
                    .disabled(revsetDraft == viewModel.revset)
                }
            }
            ToolbarItemGroup {
                Button {
                    if let id = viewModel.selectedChangeId {
                        viewModel.newChange(parent: id)
                    }
                } label: {
                    Label("New", systemImage: "plus")
                }
                .keyboardShortcut("n")
                .disabled(viewModel.selectedChangeId == nil)

                Button {
                    if let id = viewModel.selectedChangeId {
                        viewModel.abandon(rev: id)
                    }
                } label: {
                    Label("Abandon", systemImage: "trash")
                }
                .keyboardShortcut(.delete)
                .disabled(viewModel.selectedChangeId == nil)

                Button {
                    viewModel.refresh()
                } label: {
                    Label("Refresh", systemImage: "arrow.clockwise")
                }
                .keyboardShortcut("r")
            }
        }
        .safeAreaInset(edge: .bottom) {
            HStack(spacing: 12) {
                Text(viewModel.repoPath)
                    .lineLimit(1)
                    .truncationMode(.middle)
                Spacer()
                Text("\(viewModel.changes.count) changes")
                if let selected = viewModel.selectedChangeId {
                    Text("Selected \(selected)")
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                }
            }
            .font(.caption)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(.bar)
        }
        .overlay {
            if viewModel.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(.ultraThinMaterial)
            }
        }
        .alert("Error", isPresented: .init(
            get: { viewModel.error != nil },
            set: { if !$0 { viewModel.error = nil } }
        )) {
            Button("OK") { viewModel.error = nil }
        } message: {
            Text(viewModel.error ?? "")
        }
    }

    private func applyRevset() {
        let trimmed = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            return
        }
        revsetDraft = trimmed
        viewModel.applyRevset(trimmed)
    }
}
