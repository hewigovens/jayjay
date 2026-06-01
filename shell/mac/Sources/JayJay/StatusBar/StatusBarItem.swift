import Foundation

enum StatusBarItem: Identifiable {
    case text(id: String, text: String)
    case link(id: String, icon: String, text: String, url: URL, tooltip: String? = nil)
    case action(id: String, icon: String, text: String, perform: () -> Void)
    case picker(id: String, icon: String, label: String, options: [StatusBarPickerOption])

    var id: String {
        switch self {
            case let .text(id, _): id
            case let .link(id, _, _, _, _): id
            case let .action(id, _, _, _): id
            case let .picker(id, _, _, _): id
        }
    }
}

struct StatusBarPickerOption: Identifiable {
    let id: String
    let label: String
    var icon: String?
    var disabled: Bool = false
    var destructive: Bool = false
    var action: (() -> Void)?
    var children: [StatusBarPickerOption]?
}
