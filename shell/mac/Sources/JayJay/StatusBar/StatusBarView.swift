import SwiftUI

struct StatusBarView: View {
    let leadingItems: [StatusBarItem]
    let trailingItems: [StatusBarItem]

    var body: some View {
        HStack(spacing: 0) {
            ForEach(Array(leadingItems.enumerated()), id: \.element.id) { index, item in
                if index > 0 { separator }
                StatusBarItemView(item: item)
            }
            Spacer()
            ForEach(Array(trailingItems.enumerated()), id: \.element.id) { index, item in
                if index > 0 { separator }
                StatusBarItemView(item: item)
            }
        }
        .jayjayFont(11)
        .foregroundStyle(.secondary)
        .padding(.horizontal, 12)
        .padding(.vertical, 5)
        .background(.bar)
    }

    private var separator: some View {
        Text("·")
            .foregroundStyle(.quaternary)
            .padding(.horizontal, 4)
    }
}
