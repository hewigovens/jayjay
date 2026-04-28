public enum DiffGutterCheckboxState {
    case selected
    case unselected
}

public protocol DiffGutterContextActions {
    var currentSelectedLineRange: ClosedRange<Int>? { get }

    func didSelectLines(_ lineRange: ClosedRange<Int>)
}

public protocol DiffGutterSelectionActions: DiffGutterContextActions {
    func selectFile()
    func selectChangeGroup(_ lineRange: ClosedRange<Int>)
    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState?
    func toggleLineCheckbox(_ lineNumber: Int)
}

public protocol DiffGutterEditActions: DiffGutterContextActions {
    var canOpenDiffEdit: Bool { get }
    var canAbandonSelectedLines: Bool { get }

    func openDiffEdit()
    func abandonSelectedLines(in lineRange: ClosedRange<Int>)
}
