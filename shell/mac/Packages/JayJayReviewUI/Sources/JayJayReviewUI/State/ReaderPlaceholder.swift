import JayJayCore

public enum ReaderPlaceholder: Equatable, Sendable {
    case tooLarge
    case noText(ReviewFileSummary)

    public var text: String {
        switch self {
            case .tooLarge: Localized.Web.fileTooLarge
            case let .noText(file):
                file.notice ?? (file.kind == .added || file.kind == .removed ? Localized.Web.emptyFile : Localized.Web.noTextChanges)
        }
    }
}
