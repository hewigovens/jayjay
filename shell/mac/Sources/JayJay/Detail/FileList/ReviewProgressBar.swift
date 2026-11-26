import SwiftUI

/// The header's count text already carries the accessible value.
struct ReviewProgressBar: View {
    static let height: CGFloat = 2

    let reviewed: Int
    let total: Int

    private var fraction: CGFloat {
        min(CGFloat(reviewed) / CGFloat(total), 1)
    }

    var body: some View {
        Rectangle()
            .fill(Color(nsColor: .separatorColor))
            .overlay(alignment: .leading) {
                GeometryReader { proxy in
                    Rectangle()
                        .fill(FileRowReviewChrome.reviewed.tint)
                        .frame(width: proxy.size.width * fraction)
                }
            }
            .frame(height: Self.height)
            .accessibilityHidden(true)
    }
}
