import JayJayCore

enum ConvergeItem: String, CaseIterable {
    case description
    case contents
    case author
    case parents

    var title: String {
        switch self {
            case .description: Localized.Converge.description
            case .contents: Localized.Converge.contents
            case .author: Localized.Converge.author
            case .parents: Localized.Converge.parents
        }
    }
}
