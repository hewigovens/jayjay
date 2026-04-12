import Foundation

enum StatusBarItem: Identifiable {
    case text(id: String, text: String)
    case link(id: String, icon: String, text: String, url: URL, tooltip: String? = nil)
    case action(id: String, icon: String, text: String, perform: () -> Void)
    case picker(id: String, icon: String, label: String, options: [StatusBarPickerOption])

    var id: String {
        switch self {
            case .text(let id, _): id
            case .link(let id, _, _, _, _): id
            case .action(let id, _, _, _): id
            case .picker(let id, _, _, _): id
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
