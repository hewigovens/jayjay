import JayJayCore

public enum DiffPlaceholder {
    private static let nonEditablePrefixes = [
        "<binary file",
        "<directory>",
        "<git lfs ",
        "<git submodule",
        "<conflict",
        "<access denied"
    ]

    public static func isEditableText(_ text: String?) -> Bool {
        guard let text else { return true }
        return !nonEditablePrefixes.contains(where: text.hasPrefix)
    }

    public static func isGitLfs(_ text: String?) -> Bool {
        hasPrefix(text, "<git lfs ")
    }

    public static func isGitSubmodule(_ text: String?) -> Bool {
        hasPrefix(text, "<git submodule")
    }

    private static func hasPrefix(_ text: String?, _ prefix: String) -> Bool {
        text?.hasPrefix(prefix) == true
    }
}

public extension DiffHunk {
    var isSubmodulePlaceholder: Bool {
        DiffPlaceholder.isGitSubmodule(oldContent) || DiffPlaceholder.isGitSubmodule(newContent)
    }

    var isGitLfsPlaceholder: Bool {
        DiffPlaceholder.isGitLfs(oldContent) || DiffPlaceholder.isGitLfs(newContent)
    }
}
