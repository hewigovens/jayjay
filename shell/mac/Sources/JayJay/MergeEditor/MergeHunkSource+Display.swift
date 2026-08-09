import JayJayCore

extension MergeHunkSource {
    var label: String {
        switch self {
            case .left: "Left"
            case .base: "Base"
            case .right: "Right"
        }
    }
}
