import JayJayCore

extension MergeHunkSource {
    var pane: MergePane {
        switch self {
            case .left: .left
            case .base: .base
            case .right: .right
        }
    }

    var label: String {
        switch self {
            case .left: Localized.Repo.left
            case .base: Localized.Repo.base
            case .right: Localized.Repo.right
        }
    }

    var slug: String {
        switch self {
            case .left: "left"
            case .base: "base"
            case .right: "right"
        }
    }

    var useHelp: String {
        switch self {
            case .left: Localized.Merge.useLeftForThisConflict
            case .base: Localized.Merge.useBaseForThisConflict
            case .right: Localized.Merge.useRightForThisConflict
        }
    }
}
