import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                VStack(spacing: 6) {
                    HStack(spacing: 6) {
                        TextField("Revset expression", text: $revsetDraft)
                            .textFieldStyle(.roundedBorder).jayjayFont(12, design: .monospaced)
                            .onSubmit { applyRevset() }
                        Button { applyRevset() } label: {
                            Image(systemName: "arrow.right.circle.fill").foregroundStyle(.secondary)
                        }
                        .buttonStyle(.plain).disabled(revsetDraft == viewModel.revset)
                        Button {
                            revsetDraft = ""
                            applyRevset()
                        } label: {
                            Image(systemName: "xmark.circle.fill").foregroundStyle(.tertiary)
                        }
                        .buttonStyle(.plain)
                        .help("Reset to default")
                    }
                    ScrollView(.horizontal, showsIndicators: false) {
                        HStack(spacing: 6) {
                            revsetChip("All", revset: "all()")
                            revsetChip("Mine", revset: "mine()")
                            revsetChip("Bookmarks", revset: "bookmarks()")
                            revsetChip("Trunk", revset: "trunk()")
                            revsetChip("Conflicts", revset: "conflict()")
                            revsetChip("Heads", revset: "heads(all())")
                        }
                    }
                }
                .padding(.horizontal, 12).padding(.vertical, 8)
                Divider()
            }
            DAGView(
                entries: viewModel.graphEntries,
                selectedId: viewModel.selectedChangeId,
                compareFromId: viewModel.compareFromId,
                actions: viewModel,
                onMoveBookmarkForward: { viewModel.moveBookmarkForward(name: $0) },
                onPushBookmark: { viewModel.gitPush(bookmark: $0) },
                onAbandon: { requestAbandon($0) },
                onCreateBookmark: { rev in bookmarkCreateRev = rev
                    bookmarkCreateName = ""
                },
                onLoadMore: viewModel.canLoadMore ? { viewModel.loadMore() } : nil
            )
            Divider()
            CommitBox(
                description: viewModel.workingCopyDescription,
                onCommit: { await viewModel.commit(message: $0, manageSubmodules: settings.enableGitSubmoduleSupport) },
                onGenerateMessage: { await viewModel.generateCommitMessage() },
                aiProvider: viewModel.aiProvider
            )
        }
    }

    func revsetChip(_ label: String, revset: String) -> some View {
        Button {
            revsetDraft = revset
            applyRevset()
        } label: {
            Text(label)
                .jayjayFont(11, weight: .medium)
                .padding(.horizontal, 10)
                .padding(.vertical, 4)
                .background(
                    viewModel.revset == revset
                        ? AnyShapeStyle(Color.accentColor.opacity(0.2))
                        : AnyShapeStyle(Color.primary.opacity(0.06)),
                    in: Capsule()
                )
        }
        .buttonStyle(.plain)
    }

    func applyRevset() {
        let t = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        if t.isEmpty {
            let defaultRevset = RepoViewModel.buildDefaultRevset()
            revsetDraft = defaultRevset
            viewModel.applyRevset(defaultRevset)
        } else {
            revsetDraft = t
            viewModel.applyRevset(t)
        }
    }

    var statusBar: some View {
        HStack(spacing: 8) {
            if viewModel.workspaces.count > 1,
               let current = viewModel.workspaces.first(where: \.isCurrent)
            {
                Image(systemName: "square.on.square")
                    .jayjayFont(10)
                    .foregroundStyle(.secondary)
                Menu {
                    ForEach(viewModel.workspaces, id: \.name) { ws in
                        if ws.isCurrent {
                            Button {} label: {
                                Label(ws.name, systemImage: "checkmark")
                            }
                            .disabled(true)
                        } else {
                            Menu(ws.name) {
                                Button("Open") {
                                    windowManager.openRepo(ws.path)
                                }
                                if ws.name != "default" {
                                    Divider()
                                    Button("Forget") {
                                        viewModel.workspaceForget(name: ws.name)
                                        settings.removeRecentRepo(ws.path)
                                    }
                                    Button("Forget & Delete from Disk", role: .destructive) {
                                        viewModel.workspaceForget(name: ws.name)
                                        settings.removeRecentRepo(ws.path)
                                        try? FileManager.default.removeItem(atPath: ws.path)
                                    }
                                }
                            }
                        }
                    }
                } label: {
                    Text(current.name)
                        .jayjayFont(11, weight: .medium, design: .monospaced)
                        .lineLimit(1)
                }
                .menuStyle(.borderlessButton)
                .fixedSize()
                Text("·").foregroundStyle(.quaternary)
            }
            Text(viewModel.repoPath).lineLimit(1).truncationMode(.middle)
            Spacer()
            Text("\(viewModel.changes.count) changes")
        }
        .jayjayFont(11).foregroundStyle(.secondary)
        .padding(.horizontal, 12).padding(.vertical, 5).background(.bar)
    }
}
