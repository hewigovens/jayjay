import JayJayCore

extension FileKind {
    init(path: String) {
        self = fileKindForPath(path: path)
    }

    var symbolName: String {
        switch self {
            case .code: "chevron.left.forwardslash.chevron.right"
            case .shell: "terminal"
            case .text: "doc.text"
            case .data: "curlybraces"
            case .config: "gearshape"
            case .lockfile: "lock"
            case .image: "photo"
            case .table: "tablecells"
            case .notebook: "book.closed"
            case .archive: "doc.zipper"
            case .other: "doc"
        }
    }
}
