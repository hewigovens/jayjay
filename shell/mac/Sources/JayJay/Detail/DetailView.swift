import JayJayCore
import SwiftUI

struct DetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail?
    let actions: (any ChangeActions & DAGActions)?
    let onEditDescription: (String, String) -> Void
    let reviewStore: ReviewStore
    let diffStore: DiffStore
    var comparison: Comparison?
    var onClearCompare: (() -> Void)?
    var onReverseCompare: (() -> Void)?
    var onRevealChangeInDag: ((String) -> Void)?
    var onResolveDivergent: ((String) -> Void)?
    var divergentVersionCount = 0
    @Binding var activePane: ActivePane
    var evolog: EvologState?
    var evologTarget: ChangeInfo?
    var isRestoringEvologVersion = false
    var onRestoreEvologVersion: ((String) -> Void)?
    var onDismissEvolog: (() -> Void)?
    var conflictedBookmarkNames: Set<String> = []
    var selectionWithoutDiffCount = 0
    var onInteractionStateChanged: (Bool) -> Void = { _ in }

    var body: some View {
        if let evolog, let entries = evolog.entries {
            EvologView(
                entries: entries,
                changeId: evolog.rev,
                repo: repo,
                diffStore: diffStore,
                target: evologTarget,
                isRestoring: isRestoringEvologVersion,
                onRestore: { onRestoreEvologVersion?($0) },
                onDismiss: { onDismissEvolog?() }
            )
            // EvologViewModel copies its entries at init, so a reload after a restore must remount it.
            .id("\(evolog.rev)-\(entries.first?.commitId.id ?? "")")
        } else if selectionWithoutDiffCount > 1 {
            ContentUnavailableView(
                Localized.Repo.changesSelectedCount(count: selectionWithoutDiffCount),
                systemImage: "square.stack.3d.up.slash",
                description: Text(
                    Localized.Repo.theseChangesDontFormASingleLinear
                )
            )
            .accessibilityIdentifier(AID.Detail.selectionWithoutDiff)
        } else if let detail {
            ChangeDetailView(
                repoPath: repoPath, repo: repo, detail: detail,
                actions: actions, onEditDescription: onEditDescription,
                reviewStore: reviewStore, diffStore: diffStore,
                comparison: comparison,
                onClearCompare: onClearCompare,
                onReverseCompare: onReverseCompare,
                onRevealChangeInDag: onRevealChangeInDag,
                onResolveDivergent: onResolveDivergent,
                divergentVersionCount: divergentVersionCount,
                activePane: $activePane,
                conflictedBookmarkNames: conflictedBookmarkNames,
                onInteractionStateChanged: onInteractionStateChanged
            )
            .id("\(detail.info.selectionRevision)|\(comparison?.fromId ?? "")")
        } else {
            ContentUnavailableView(
                Localized.Repo.selectAChangeTitle, systemImage: "doc.text",
                description: Text(Localized.Repo.chooseAChangeFromTheList)
            )
        }
    }
}
