import JayJayCore
import SwiftUI

/// The change's diffs in one scroll: every file when none is selected, or just the selected one.
struct AllFilesDiff<Section: View>: View {
    let hunks: [DiffHunk]
    /// Reports whether the diffs have scrolled away from the top, so the change header can shrink to its compact form.
    let onScrolledChange: (Bool) -> Void
    let section: (DiffHunk) -> Section

    var body: some View {
        ScrollView {
            LazyVStack(alignment: .leading, spacing: 0) {
                ForEach(hunks, id: \.path) { hunk in
                    section(hunk)
                    Divider()
                }
            }
        }
        .onScrollGeometryChange(for: Bool.self) { $0.contentOffset.y > 24 } action: { _, scrolled in
            onScrolledChange(scrolled)
        }
    }
}
