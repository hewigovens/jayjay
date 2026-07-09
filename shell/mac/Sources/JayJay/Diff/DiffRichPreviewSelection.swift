struct DiffRichPreviewSelection: Equatable {
    enum Kind {
        case svg
        case markdown
        case html
        case projection
    }

    let kind: Kind
    let path: String

    func isActive(_ kind: Kind, path: String) -> Bool {
        self.kind == kind && self.path == path
    }
}
