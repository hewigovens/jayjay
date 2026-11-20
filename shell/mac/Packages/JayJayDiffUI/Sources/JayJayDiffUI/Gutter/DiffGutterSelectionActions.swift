import JayJayCore

public protocol DiffGutterSelectionActions: DiffGutterContextActions {
    func selectFile()
    func changeGroupActionTitle(for lineRange: ClosedRange<Int>) -> String
    func selectChangeGroup(_ lineRange: ClosedRange<Int>)
    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState?
    func toggleLineCheckbox(_ lineNumber: Int)
}

public extension DiffGutterSelectionActions {
    func changeGroupActionTitle(for lineRange: ClosedRange<Int>) -> String {
        Localized.Diff.selectChangeGroup
    }
}
