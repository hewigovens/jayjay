import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                BookmarkPicker(bookmarks: viewModel.bookmarks, actions: viewModel)
                Spacer(minLength: 8)
                Text("\(viewModel.changes.count) \(viewModel.changes.count == 1 ? "change" : "changes")")
                    .jayjayFont(12)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }
            .padding(.leading, 8)
            .padding(.trailing, 14)
            .frame(minHeight: PaneLayout.headerHeight)
            Divider()
            if let name = viewModel.pendingPushBookmark {
                pushFollowUpBanner(name)
                Divider()
            }
            DAGView(
                entries: viewModel.graphEntries,
                layout: viewModel.dagLayout,
                capabilities: viewModel.selectionCapabilities,
                graphGeneration: viewModel.graphGeneration,
                selectedId: viewModel.selectedChangeId,
                selectedIds: viewModel.selectedChangeIds,
                compareFromId: viewModel.compareFromId,
                actions: viewModel,
                onRequest: { handleDAGRequest($0) },
                activePane: Bindable(keyboardFocus).activePane,
                revealRequest: viewModel.dagRevealRequest,
                prHostName: viewModel.prHostName,
                conflictedBookmarkNames: viewModel.conflictedBookmarkNames,
                workspacesByName: viewModel.workspacesByName
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
                    onGenerateMessage: {
                        await viewModel.generateCommitMessage(using: settings.aiProviderOrder)
                    },
                    aiProvider: viewModel.aiProvider
                )
                .task(id: settings.aiProviderOrder) {
                    let label = await settings.aiProviderOrder.firstReadyLabel()
                    if !Task.isCancelled {
                        viewModel.aiProvider = label
                    }
                }
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

    private var shouldShowCommitBox: Bool {
        viewModel.selectedChange?.info.isWorkingCopy == true
    }
}
