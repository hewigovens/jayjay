import SwiftUI

struct PickerRow: Identifiable {
    let id: String
    let searchText: String
    let height: CGFloat
    let action: (() -> Void)?
    let content: (_ highlighted: Bool) -> AnyView
    private(set) var contextMenu: (() -> AnyView)?

    init(
        id: String,
        searchText: String,
        height: CGFloat = 30,
        action: (() -> Void)? = nil,
        @ViewBuilder content: @escaping (_ highlighted: Bool) -> some View
    ) {
        self.id = id
        self.searchText = searchText
        self.height = height
        self.action = action
        self.content = { AnyView(content($0)) }
    }

    func withContextMenu(@ViewBuilder _ items: @escaping () -> some View) -> PickerRow {
        var row = self
        row.contextMenu = { AnyView(items()) }
        return row
    }
}

struct PickerSection: Identifiable {
    let id: String
    let title: String?
    let rows: [PickerRow]
}
