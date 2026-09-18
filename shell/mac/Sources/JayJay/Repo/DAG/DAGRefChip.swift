import JayJayCore

enum DAGRefChip: Hashable {
    case workingCopy
    case conflict
    case divergent
    case bookmark(String)
    case gitTag(String)
    case workspace(String)

    static func chips(for change: ChangeInfo) -> [DAGRefChip] {
        var chips: [DAGRefChip] = []
        if change.isWorkingCopy {
            chips.append(.workingCopy)
        }
        if change.hasConflict {
            chips.append(.conflict)
        }
        if change.isDivergent {
            chips.append(.divergent)
        }
        chips += change.bookmarks.map(DAGRefChip.bookmark)
        chips += change.tags.map(DAGRefChip.gitTag)
        chips += change.workspaces.map(DAGRefChip.workspace)
        return chips
    }

    var label: String {
        switch self {
            case .workingCopy: "@"
            case .conflict: "conflict"
            case .divergent: "divergent"
            case let .bookmark(name): name
            case let .gitTag(name): name
            case let .workspace(name): "\(name)@"
        }
    }
}
