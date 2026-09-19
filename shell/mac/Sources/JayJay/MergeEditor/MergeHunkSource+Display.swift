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
            case .left: "Left"
            case .base: "Base"
            case .right: "Right"
        }
    }
}
