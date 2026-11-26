import JayJayCore
import SwiftUI

extension RepoContentView {
    var sidebar: some View {
        VStack(spacing: 0) {
            HStack(spacing: 8) {
                BookmarkPicker(bookmarks: viewModel.bookmarks, isLoaded: viewModel.hasFinishedFirstLoad, actions: viewModel)
                Spacer(minLength: 8)
                let counts = viewModel.changeCounts
                if counts.divergent > 0 {
                    countBadge(.divergent, count: Int(counts.divergent))
                }
                if counts.conflicted > 0 {
                    countBadge(.conflicted, count: Int(counts.conflicted))
                }
                Text(Localized.Repo.changeCount(count: viewModel.changes.count))
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
                compareFromId: viewModel.comparison?.fromId,
                actions: viewModel,
                onRequest: { handleDAGRequest($0) },
                activePane: Bindable(keyboardFocus).activePane,
                revealRequest: viewModel.dagRevealRequest,
                trunkName: viewModel.trunkName,
                prHostName: viewModel.prHostName,
                uploadTitle: viewModel.uploadTitle,
                gerritUpdateTitle: viewModel.gerritUpdateTitle,
                conflictedBookmarkNames: viewModel.conflictedBookmarkNames,
                remoteTagNames: viewModel.remoteTagNames,
                workspacesByName: viewModel.workspacesByName,
                refreshMode: viewModel.refreshMode,
                bisect: viewModel.bisect.state,
                selectionGraph: viewModel.selectionGraph
            )
            if viewModel.selectedChange?.info.isWorkingCopy == true, viewModel.showsWorkingCopyDraft {
                Divider()
                CommitBox(viewModel: viewModel)
            }
        }
    }

    private func countBadge(_ kind: ChangeCountKind, count: Int) -> some View {
        let label = kind.label(count: count)
        return Button {
            viewModel.applyFilter(kind.revset)
        } label: {
            HStack(spacing: 2) {
                Image(systemName: kind.systemImage).imageScale(.small)
                Text(count, format: .number)
            }
            .jayjayFont(12, weight: .medium)
            .foregroundStyle(kind.tint)
            .fixedSize()
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .help(label)
        .accessibilityLabel(label)
        .accessibilityIdentifier(kind.accessibilityId)
    }

    func pushFollowUpBanner(_ name: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "bookmark.fill").foregroundStyle(.green).jayjayFont(11)
            Text(Localized.Repo.moved).jayjayFont(11).foregroundStyle(.secondary)
            Text(name).jayjayFont(11, weight: .medium, design: .monospaced).lineLimit(1)
            Spacer()
            Button(Localized.Menu.push) { viewModel.confirmPendingPush() }
                .controlSize(.small)
                .disabled(viewModel.isPushingInFlight)
            Button {
                viewModel.dismissPendingPush()
            } label: {
                Image(systemName: "xmark").jayjayFont(10)
            }
            .iconButtonStyle().foregroundStyle(.secondary)
            .help(Localized.Repo.dismiss)
        }
        .padding(.horizontal, 12).padding(.vertical, 6)
        .glassEffect(in: RoundedRectangle(cornerRadius: 8))
    }
}

private enum ChangeCountKind {
    case divergent, conflicted

    var systemImage: String {
        switch self {
            case .divergent: "arrow.triangle.branch"
            case .conflicted: "exclamationmark.triangle.fill"
        }
    }

    var tint: Color {
        switch self {
            case .divergent: .orange
            case .conflicted: .red
        }
    }

    var revset: String {
        switch self {
            case .divergent: "divergent()"
            case .conflicted: "conflicts()"
        }
    }

    var accessibilityId: String {
        switch self {
            case .divergent: AID.Sidebar.divergentCount
            case .conflicted: AID.Sidebar.conflictedCount
        }
    }

    func label(count: Int) -> String {
        switch self {
            case .divergent: Localized.Repo.divergentCount(count: count)
            case .conflicted: Localized.Repo.conflictedCount(count: count)
        }
    }
}
