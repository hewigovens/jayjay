import Foundation

enum StatusBarItem: Identifiable {
    case text(id: String, icon: String? = nil, text: String, tooltip: String? = nil)
    case link(id: String, icon: String, text: String, url: URL, tooltip: String? = nil)
    case action(id: String, icon: String, text: String, perform: () -> Void)

    var id: String {
        switch self {
            case let .text(id, _, _, _): id
            case let .link(id, _, _, _, _): id
            case let .action(id, _, _, _): id
        }
    }
}
