import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            if showRevsetFilter {
                VStack(spacing: 6) {
                    HStack(spacing: 6) {
                        if let previousAncestorFilter {
                            Button {
                                revsetDraft = previousAncestorFilter
                                applyRevset()
                            } label: {
                                Image(systemName: "arrow.left")
                            }
                            .buttonStyle(.plain)
                            .help("Back to previous filter")
                            .accessibilityLabel("Back to previous filter")
                        }
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
                            ForEach(RevsetExpressions.filterPresets, id: \.id) { preset in
                                revsetChip(preset.label, revset: preset.revset)
                            }
                        }
                    }
                }
                .padding(.horizontal, 12).padding(.vertical, 8)
                Divider()
            }
            if let name = viewModel.pendingPushBookmark {
                pushFollowUpBanner(name)
                Divider()
            }
            DAGView(
                entries: viewModel.graphEntries,
                selectedId: viewModel.selectedChangeId,
                selectedIds: viewModel.selectedChangeIds,
                compareFromId: viewModel.compareFromId,
                actions: viewModel,
                onRequestRebase: { handleDAGRebase($0) },
                activePane: $activePane,
                revealRequest: dagRevealRequest,
                prHostName: viewModel.prHostName,
                onMoveBookmarkToRev: { viewModel.moveBookmark(name: $0, toRev: $1) },
                onMoveWorkingCopyToRev: { viewModel.edit(rev: $0) },
                onPushBookmark: { viewModel.gitPush(bookmark: $0) },
                onOpenPRForBookmark: { viewModel.openPR(bookmark: $0) },
                onDeleteBookmark: { viewModel.removeBookmark(name: $0, fromRev: $1) },
                conflictedBookmarkNames: viewModel.conflictedBookmarkNames,
                onAbandon: { requestAbandon($0) },
                onAbandonSelection: { requestAbandonSelection($0) },
                onSquashSelection: { requestSquashSelection($0) },
                onCreateBookmark: { rev in presentBookmarkCreate(rev: rev) },
                onCreateStackedPRs: { rev in presentStackedPr(rev: rev) },
                onShowAncestors: { commitId in
                    if previousAncestorFilter == nil {
                        previousAncestorFilter = viewModel.revset
                    }
                    showRevsetFilter = true
                    viewModel.applyRevset(ancestorsRevset(commitId: commitId), selecting: commitId)
                },
                onLoadMore: viewModel.canLoadMore ? { viewModel.loadMore() } : nil
            )
            if shouldShowCommitBox {
                Divider()
                CommitBox(
                    description: viewModel.workingCopyDescription,
                    summary: $viewModel.commitSummaryDraft,
                    details: $viewModel.commitDescriptionDraft,
                    onSaveDescription: { viewModel.describeWorkingCopy(message: $0) },
                    onCommit: {
                        await viewModel.commit(message: $0, manageSubmodules: settings.enableGitSubmoduleSupport)
                    },
                    onGenerateMessage: { await viewModel.generateCommitMessage() },
                    aiProvider: viewModel.aiProvider
                )
            }
        }
    }

    func pushFollowUpBanner(_ name: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "bookmark.fill").foregroundStyle(.green).jayjayFont(11)
            Text("Moved").jayjayFont(11).foregroundStyle(.secondary)
            Text(name).jayjayFont(11, weight: .medium, design: .monospaced).lineLimit(1)
            Spacer()
            Button("Push") { viewModel.confirmPendingPush() }
                .controlSize(.small)
                .disabled(viewModel.isPushingInFlight)
            Button {
                viewModel.dismissPendingPush()
            } label: {
                Image(systemName: "xmark").jayjayFont(10)
            }
            .buttonStyle(.plain).foregroundStyle(.secondary)
            .help("Dismiss")
        }
        .padding(.horizontal, 12).padding(.vertical, 6)
        .glassEffect(in: RoundedRectangle(cornerRadius: 8))
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
        previousAncestorFilter = nil
        let t = revsetDraft.trimmingCharacters(in: .whitespacesAndNewlines)
        revsetDraft = t.isEmpty ? RepoViewModel.buildDefaultRevset() : t
        viewModel.applyRevset(revsetDraft)
    }

    private var shouldShowCommitBox: Bool {
        viewModel.selectedChange?.info.isWorkingCopy == true
    }
}
