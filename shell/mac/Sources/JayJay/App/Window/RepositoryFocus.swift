import SwiftUI

private struct FocusedRepoPathKey: FocusedValueKey {
    typealias Value = String
}

private struct FocusedGitFetchKey: FocusedValueKey {
    typealias Value = () -> Void
}

private struct FocusedGitPushKey: FocusedValueKey {
    typealias Value = () -> Void
}

private struct FocusedShowUndoKey: FocusedValueKey {
    typealias Value = () -> Void
}

extension FocusedValues {
    var jayjayRepoPath: String? {
        get { self[FocusedRepoPathKey.self] }
        set { self[FocusedRepoPathKey.self] = newValue }
    }

    var jayjayGitFetch: (() -> Void)? {
        get { self[FocusedGitFetchKey.self] }
        set { self[FocusedGitFetchKey.self] = newValue }
    }

    var jayjayGitPush: (() -> Void)? {
        get { self[FocusedGitPushKey.self] }
        set { self[FocusedGitPushKey.self] = newValue }
    }

    var jayjayShowUndo: (() -> Void)? {
        get { self[FocusedShowUndoKey.self] }
        set { self[FocusedShowUndoKey.self] = newValue }
    }
}
