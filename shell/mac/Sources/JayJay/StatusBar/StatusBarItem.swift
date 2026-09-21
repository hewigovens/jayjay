import Foundation

enum StatusBarItem: Identifiable {
    case text(id: String, icon: String? = nil, text: String, tooltip: String? = nil, shrinks: Bool = false)
    case link(id: String, icon: String, text: String, url: URL, tooltip: String? = nil)
    case action(id: String, icon: String, text: String, tooltip: String? = nil, shrinks: Bool = false, perform: () -> Void)

    var id: String {
        switch self {
            case let .text(id, _, _, _, _): id
            case let .link(id, _, _, _, _): id
            case let .action(id, _, _, _, _, _): id
        }
    }

    /// Items with open-ended text give up width first, so the short status items stay whole.
    var shrinks: Bool {
        switch self {
            case let .text(_, _, _, _, shrinks): shrinks
            case .link: false
            case let .action(_, _, _, _, shrinks, _): shrinks
        }
    }
}
