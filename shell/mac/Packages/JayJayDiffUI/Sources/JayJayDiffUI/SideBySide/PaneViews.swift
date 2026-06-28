import AppKit

/// Bundles the AppKit views and layout managers needed to render one SBS pane.
struct PaneViews {
    let container: DiffTextContainerView
    let textView: NSTextView
    let gutterTextView: DiffGutterTextView
    let textLayout: DiffLayoutManager
    let gutterLayout: DiffLayoutManager
}

extension DiffTextContainerView {
    /// `nil` if either layout manager is missing or the wrong subclass.
    func paneViews() -> PaneViews? {
        guard let textLayout = textView.layoutManager as? DiffLayoutManager,
              let gutterLayout = gutterTextView.layoutManager as? DiffLayoutManager
        else { return nil }
        return PaneViews(
            container: self,
            textView: textView,
            gutterTextView: gutterTextView,
            textLayout: textLayout,
            gutterLayout: gutterLayout
        )
    }
}
