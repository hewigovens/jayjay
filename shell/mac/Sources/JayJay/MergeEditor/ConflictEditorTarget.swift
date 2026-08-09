import JayJayCore

struct ConflictEditorTarget: Sendable {
    let repo: JayJayRepo
    let rev: String
    let path: String

    func load() throws -> ConflictEditorData {
        try repo.conflictEditor(rev: rev, path: path)
    }
}
