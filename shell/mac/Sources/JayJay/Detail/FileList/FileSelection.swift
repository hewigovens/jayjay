import JayJayCore

struct FileSelection: Equatable {
    var primary: String?
    var paths: Set<String> = []
    var anchor: String?

    static func single(_ path: String) -> FileSelection {
        FileSelection(primary: path, paths: [path], anchor: path)
    }

    var ordered: OrderedSelection {
        orderedSelection(selected: Array(paths), primary: primary, anchor: anchor)
    }

    func preserved(in available: Set<String>) -> FileSelection {
        guard let primary, available.contains(primary) else { return FileSelection() }
        let kept = paths.intersection(available)
        return FileSelection(
            primary: primary,
            paths: kept.isEmpty ? [primary] : kept,
            anchor: anchor.flatMap { available.contains($0) ? $0 : nil } ?? primary
        )
    }
}

extension FileSelection {
    init(_ selection: OrderedSelection) {
        self.init(primary: selection.primary, paths: Set(selection.selected), anchor: selection.anchor)
    }
}
