public protocol DiffGutterNoteActions: DiffGutterContextActions {
    var reviewNotesEnabled: Bool { get }

    func activeNotes(anchor: DiffReviewNoteAnchor) -> [DiffReviewNoteSummary]
    func addNote(anchor: DiffReviewNoteAnchor)
    func editNote(id: String)
    func deleteNote(id: String)
    func resolveNote(id: String)
}
