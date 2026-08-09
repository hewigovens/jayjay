import JayJayCore

/// What the right-hand detail pane is showing. Mutually exclusive — replaces
/// the previous mix of `annotateLines`/`annotatePath`/`fileHistory`/`fileHistoryPath`/`isDiffEditMode`.
enum DetailPaneMode {
    /// Default file list + diff section.
    case files
    case annotate(lines: [AnnotationLine], path: String)
    case fileHistory(history: [ChangeInfo], path: String)
    case diffEdit

    var isFiles: Bool {
        if case .files = self {
            true
        } else {
            false
        }
    }

    var isDiffEdit: Bool {
        if case .diffEdit = self {
            true
        } else {
            false
        }
    }
}
