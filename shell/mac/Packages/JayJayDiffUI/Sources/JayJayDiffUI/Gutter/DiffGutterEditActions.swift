public protocol DiffGutterEditActions: DiffGutterContextActions {
    var canOpenDiffEdit: Bool { get }
    var canAbandonSelectedLines: Bool { get }
    var canEditFile: Bool { get }
    /// Nil when no external editor can open at a line.
    var externalEditorName: String? { get }

    func openDiffEdit()
    func abandonSelectedLines(in lineRange: ClosedRange<Int>)
    func editFile(atLine line: UInt32)
    func openInExternalEditor(atLine line: UInt32)
}
