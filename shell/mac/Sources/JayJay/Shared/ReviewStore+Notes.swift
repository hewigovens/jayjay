import JayJayCore

extension ReviewStore {
    func listNotes(changeId: String, includeResolved: Bool = false) -> [ReviewNote] {
        // The snapshot always keeps resolved notes — the diff gutter renders them as dimmed history markers — so only the returned list honors the filter.
        let loaded = reviewListNotes(
            changeId: changeId,
            includeResolved: true,
            storePath: storePath
        )
        notes = loaded
        return includeResolved ? loaded : loaded.filter { !$0.resolved }
    }

    @discardableResult
    func addNote(anchor: NoteAnchor, body: String) -> ReviewNote {
        let note = reviewAddNote(anchor: anchor, body: body, storePath: storePath)
        replace(note)
        return note
    }

    @discardableResult
    func updateNote(id: String, body: String) -> ReviewNote? {
        guard let note = reviewUpdateNote(id: id, body: body, storePath: storePath) else {
            return nil
        }
        replace(note)
        return note
    }

    func deleteNote(id: String) {
        if reviewDeleteNote(id: id, storePath: storePath) {
            notes.removeAll { $0.id == id }
        }
    }

    func resolveNote(id: String) {
        guard let note = reviewResolveNote(id: id, storePath: storePath) else { return }
        replace(note)
    }

    private func replace(_ note: ReviewNote) {
        if let index = notes.firstIndex(where: { $0.id == note.id }) {
            notes[index] = note
        } else {
            notes.append(note)
        }
    }
}
