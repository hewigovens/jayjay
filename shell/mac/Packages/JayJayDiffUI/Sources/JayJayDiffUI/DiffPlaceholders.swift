import JayJayCore

/// Nil-handling facade over jayjay-core's placeholder predicates (`crate::placeholder`),
/// so the editable-vs-placeholder rule cannot drift from core.
public enum DiffPlaceholder {
    public static func isEditableText(_ text: String?) -> Bool {
        guard let text else { return true }
        return isEditableDiffText(text: text)
    }

    public static func isGitLfs(_ text: String?) -> Bool {
        guard let text else { return false }
        return isGitLfsPlaceholder(text: text)
    }

    public static func isGitSubmodule(_ text: String?) -> Bool {
        guard let text else { return false }
        return isGitSubmodulePlaceholder(text: text)
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
