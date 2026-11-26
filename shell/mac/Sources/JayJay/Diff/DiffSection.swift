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
    var onEditFile: ((UInt32?) -> Void)?
    var onOpenInExternalEditor: ((UInt32) -> Void)?
    var onReviewStateChanged: (() -> Void)?
    var onReviewSnapshotLoaded: ((DiffHunk, ReviewFileSnapshot?) -> Void)?
    var fileReview: FileRowReviewChrome = .hidden
    var onToggleFileReview: (() -> Void)?
    var compareFromRev: String?
    var lineStats: FileDiffStats?
    var fitsContent = false
    /// The change header shows the layout toggle and Edit Diff itself, since they apply to every file.
    var showsSharedActions = true
    /// Set in a stack of files, where a large diff folds to a row that opens it alone.
    var onShowAlone: (() -> Void)?
    var stackedHeights: StackedDiffHeights?
    var isCollapsed = false
    var onToggleCollapse: (() -> Void)?
    /// Called when the file folds while its header is pinned, since the rows that held it in view are gone.
    var onRevealHeader: (() -> Void)?

    // Non-private members are read by the DiffSection+Content / +EditActions / +ReviewActions extensions.
    // Display lines and change groups are computed once per loaded diff; updateNSView re-runs per observed change and the FFI is O(diff bytes).
    @State var loadedDiff: DiffSectionLoadedDiff?
    @State var isComputing = false
    @State var loadError: String?
    @State var loadAttempt = 0
    @State var selectedLineRange: ClosedRange<Int>?
    @State var contextExpansion = ContextExpansionSession()
    @State var contextExpansionDisplay = DiffContextExpansionDisplay()
    @State var richPreviewSelection: DiffRichPreviewSelection?
    @State var compactHeader = false
    @State var fittedDiffHeight: CGFloat?
    @State private var pinnedHeaderOffset: CGFloat = 0
    @Environment(AppSettings.self) var settings
    @Environment(DiffCommands.self) private var diffCommands: DiffCommands?
    @Environment(\.jayjayTypography) private var typography
    @Environment(\.colorScheme) var colorScheme
    @Environment(\.appThemes) var appThemes

    var headerHeight: CGFloat {
        PaneLayout.fileRowHeight(scale: typography.interfaceScale)
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            diffHeader
                .padding(.horizontal, PaneLayout.detailInset)
                .frame(minHeight: headerHeight)
                .background(pinnedHeaderBackground)
                .overlay(alignment: .bottom) { Divider().opacity(pinnedHeaderOffset > 0 ? 1 : 0) }
                .offset(y: pinnedHeaderOffset)
                .zIndex(1)
            if !isCollapsed {
                if let message = contextExpansionDisplay.errorMessage {
                    expansionErrorBanner(message)
                        .padding(.bottom, 8)
                }
                // Without the stack, a diff that has nothing to show yet has no view to take the height.
                ZStack { diffContent }
                    .frame(height: fitsContent ? stackedContentHeight : nil)
                    .onGeometryChange(for: CGFloat.self) { $0.size.height } action: { stackedHeights?.record($0, of: hunk.path, in: heightBasis) }
                    .onChange(of: heightBasis) { _, basis in stackedHeights?.carry(hunk.path, to: basis) }
            }
        }
        .onChange(of: isCollapsed) { _, collapsed in
            if collapsed, pinnedHeaderOffset > 0 {
                onRevealHeader?()
            }
        }
        // A stacked file's header stays at the top of the list while its diff scrolls under it, until the file's end carries it away.
        .onGeometryChange(for: CGFloat.self) { proxy in
            guard fitsContent, let visible = proxy.bounds(of: .scrollView) else { return 0 }
            return min(max(visible.minY, 0), max(proxy.size.height - headerHeight, 0))
        } action: { pinnedHeaderOffset = $0 }
        .onChange(of: hunk.path) { _, _ in
            resetRichViewState()
        }
        .onChange(of: diffCommands?.expandAllContextRequest) { _, _ in
            expandAllContext()
        }
        .task(id: "\(compareFromRev ?? "")|\(rev ?? "")|\(hunk.path)|\(settings.ignoreWhitespace)|\(settings.language ?? "")|\(projectionModeKey)|\(loadAttempt)") {
            await computeDiffAsync()
        }
        .environment(\.diffTypography, typography.diffTypography)
    }

    /// A tinted window has no reading background to cover the rows a pinned header sits over, so it takes a material.
    private var pinnedHeaderBackground: AnyShapeStyle {
        guard fitsContent else { return AnyShapeStyle(.clear) }
        return settings.tintsWindow(in: colorScheme) ? AnyShapeStyle(.regularMaterial) : AnyShapeStyle(settings.readingBackground(colorScheme))
    }

    private func expansionErrorBanner(_ message: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.triangle")
                .foregroundStyle(.secondary)
            Text(message)
                .jayjayFont(12)
                .foregroundStyle(.secondary)
            Spacer(minLength: 8)
            Button(Localized.Repo.dismiss) { contextExpansionDisplay.errorMessage = nil }
                .buttonStyle(.plain)
                .jayjayFont(12)
                .foregroundStyle(Color.accentColor)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 8, style: .continuous))
        .padding(.horizontal, PaneLayout.detailInset)
    }
}
