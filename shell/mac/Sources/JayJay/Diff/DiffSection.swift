import JayJayCore
import JayJayDiffUI
import SwiftUI

struct DiffSection: View {
    let hunk: DiffHunk
    let rev: String?
    var commitId: String?
    // Review store key: always the real change id. `rev` is the selection revision, which is a commit id for divergent changes and would hide notes from CLI/core reconciliation (they key by change id).
    var reviewChangeId: String?
    let repo: JayJayRepo?
    let actions: (any ChangeActions & DAGActions)?
    let isWorkingCopy: Bool
    let diffStore: DiffStore
    let reviewStore: ReviewStore?
    /// Stale/orphaned note ids from the async reconciliation report; their bubbles must not expand into the diff since their anchors may be wrong.
    var staleNoteIds: Set<String> = []
    // Owned by ChangeDetailView: this view is rebuilt on every commit-id change (background snapshots included), which would reset a local @State editor and dismiss the sheet mid-typing.
    @Binding var noteEditor: ReviewNoteEditorState?
    var onOpenDiffEdit: (() -> Void)?
    var onReviewStateChanged: (() -> Void)?
    var compareFromRev: String?

    // Non-private members are read by the DiffSection+Content / +EditActions / +ReviewActions extensions.
    // Display lines and change groups are computed once per loaded diff; updateNSView re-runs per observed change and the FFI is O(diff bytes).
    @State var loadedDiff: DiffSectionLoadedDiff?
    @State var isComputing = false
    @State var selectedLineRange: ClosedRange<Int>?
    @State var richPreviewSelection: DiffRichPreviewSelection?
    @Environment(AppSettings.self) var settings
    @Environment(\.jayjayFontSize) private var jayjayFontSize
    @Environment(\.jayjayFontFamily) private var jayjayFontFamily

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            diffHeader
            diffContent
        }
        .accessibilityIdentifier(AID.Diff.section)
        .onChange(of: hunk.path) { _, _ in
            resetRichViewState()
        }
        .task(id: "\(compareFromRev ?? "")|\(rev ?? "")|\(hunk.path)|\(settings.ignoreWhitespace)|\(projectionModeKey)") {
            await computeDiffAsync()
        }
        .environment(\.diffFontSize, jayjayFontSize)
        .environment(\.diffFontFamily, jayjayFontFamily.nsFontName)
    }
}
