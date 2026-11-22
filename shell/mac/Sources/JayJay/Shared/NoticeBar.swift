import SwiftUI

struct NoticeBar<Content: View, Footer: View>: View {
    let systemImage: String
    let tint: Color
    let title: String
    @ViewBuilder let content: Content
    @ViewBuilder let footer: Footer

    var body: some View {
        VStack(spacing: 0) {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 10) {
                    Image(systemName: systemImage)
                        .foregroundStyle(tint)
                    Text(title)
                        .jayjayFont(12, weight: .semibold)
                    content
                }
                footer
            }
            .controlSize(.small)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.horizontal, PaneLayout.detailInset)
            .padding(.vertical, 8)
            .background(tint.opacity(0.08))
            Divider()
        }
    }
}

extension NoticeBar where Footer == EmptyView {
    init(systemImage: String, tint: Color, title: String, @ViewBuilder content: () -> Content) {
        self.init(systemImage: systemImage, tint: tint, title: title, content: content) {}
    }
}

extension NoticeBar where Content == EmptyView, Footer == EmptyView {
    init(systemImage: String, tint: Color, title: String) {
        self.init(systemImage: systemImage, tint: tint, title: title) {}
    }
}
