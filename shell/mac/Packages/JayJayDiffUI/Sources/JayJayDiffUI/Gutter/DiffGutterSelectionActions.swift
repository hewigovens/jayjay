public protocol DiffGutterSelectionActions: DiffGutterContextActions {
    func selectFile()
    func selectChangeGroup(_ lineRange: ClosedRange<Int>)
    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState?
    func toggleLineCheckbox(_ lineNumber: Int)
}
