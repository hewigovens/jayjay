public protocol DiffGutterEditActions: DiffGutterContextActions {
    var canOpenDiffEdit: Bool { get }
    var canAbandonSelectedLines: Bool { get }

    func openDiffEdit()
    func abandonSelectedLines(in lineRange: ClosedRange<Int>)
}
