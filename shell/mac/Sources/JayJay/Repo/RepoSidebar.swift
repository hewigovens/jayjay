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
                onAbandon: { requestAbandon($0) },
                onCreateBookmark: { rev in bookmarkCreateRev = rev
                    bookmarkCreateName = ""
                },
                onLoadMore: viewModel.isCustomRevset || !viewModel.hasMoreToLoad ? nil : { viewModel.loadMore() }
            )
            Divider()
            CommitBox(
                description: viewModel.workingCopyDescription,
                onCommit: { viewModel.commit(message: $0) },
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
            viewModel.ancestorLimit = 20
            let defaultRevset = RepoViewModel.buildDefaultRevset(limit: 20)
            revsetDraft = defaultRevset
            viewModel.applyRevset(defaultRevset)
        } else {
            revsetDraft = t
            viewModel.applyRevset(t)
        }
    }

    var statusBar: some View {
        HStack(spacing: 12) {
            Text(viewModel.repoPath).lineLimit(1).truncationMode(.middle)
            Spacer()
            Text("\(viewModel.changes.count) changes")
        }
        .jayjayFont(11).foregroundStyle(.secondary)
        .padding(.horizontal, 12).padding(.vertical, 5).background(.bar)
    }
}
