import JayJayCore
import SwiftUI

struct DetailView: View {
    let repoPath: String
    let repo: JayJayRepo?
    let detail: ChangeDetail?
    let actions: (any ChangeActions & DAGActions)?
    let onDescribe: (String, String) -> Void
    let reviewStore: ReviewStore
    let diffStore: DiffStore
    var compareFromId: String?
    var compareDisplay: CompareDisplay?
    var onClearCompare: (() -> Void)?
    var onReverseCompare: (() -> Void)?
    var onRevealChangeInDag: ((String) -> Void)?
    @Binding var activePane: ActivePane
    var evologEntries: [EvologEntry]?
    var evologRev: String?
    var onDismissEvolog: (() -> Void)?

    var body: some View {
        if let entries = evologEntries, let rev = evologRev {
            EvologView(
                entries: entries,
                changeId: rev,
                repo: repo,
                diffStore: diffStore,
                onDismiss: { onDismissEvolog?() }
            )
            .id(rev)
        } else if let detail {
            ChangeDetailView(
                repoPath: repoPath, repo: repo, detail: detail,
                actions: actions, onDescribe: onDescribe,
                reviewStore: reviewStore, diffStore: diffStore,
                compareFromId: compareFromId,
                compareDisplay: compareDisplay,
                onClearCompare: onClearCompare,
                onReverseCompare: onReverseCompare,
                onRevealChangeInDag: onRevealChangeInDag,
                activePane: $activePane
            )
            .id("\(detail.info.selectionRevision)|\(compareFromId ?? "")")
        } else {
            ContentUnavailableView(
                "Select a Change", systemImage: "doc.text",
                description: Text("Choose a change from the list to see its details.")
            )
        }
    }
}
