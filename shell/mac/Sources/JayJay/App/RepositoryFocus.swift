import SwiftUI

private struct FocusedRepoPathKey: FocusedValueKey {
    typealias Value = String
}

extension FocusedValues {
    var jayjayRepoPath: String? {
        get { self[FocusedRepoPathKey.self] }
        set { self[FocusedRepoPathKey.self] = newValue }
    }
}
