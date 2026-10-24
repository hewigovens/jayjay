import JayJayCore

extension Int {
    var reviewNoteCountLabel: String {
        Localized.Repo.reviewNoteCount(count: self)
    }
}
