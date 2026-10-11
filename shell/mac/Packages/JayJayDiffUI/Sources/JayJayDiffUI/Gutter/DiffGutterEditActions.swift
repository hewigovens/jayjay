public protocol DiffGutterEditActions: DiffGutterContextActions {
    var canOpenDiffEdit: Bool { get }
    var canAbandonSelectedLines: Bool { get }
    var canEditFile: Bool { get }

    func openDiffEdit()
    func abandonSelectedLines(in lineRange: ClosedRange<Int>)
    func editFile(atLine line: UInt32)
}
