import JayJayCore

/// What the right-hand detail pane is showing. Mutually exclusive — replaces
/// the previous mix of `annotateLines`/`annotatePath`/`isDiffEditMode`.
enum DetailPaneMode {
    /// Default file list + diff section.
    case files
    case annotate(lines: [AnnotationLine], path: String, error: String? = nil)
    case diffEdit

    var isDiffEdit: Bool {
        if case .diffEdit = self {
            true
        } else {
            false
        }
    }
}
