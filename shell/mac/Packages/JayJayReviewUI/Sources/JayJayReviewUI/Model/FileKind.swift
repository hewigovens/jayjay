import JayJayCore

public enum FileKind: String, Codable, Sendable, CaseIterable {
    case added, removed, modified, renamed, copied

    public var label: String {
        switch self {
            case .added: Localized.Diff.added
            case .removed: Localized.Diff.removed
            case .modified: Localized.Repo.modified
            case .renamed: Localized.Diff.renamed
            case .copied: Localized.Diff.copied
        }
    }
}
