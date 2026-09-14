import Foundation

enum DescriptionHeight {
    static let collapsed: CGFloat = 80

    /// The expanded body still leaves most of the pane to the diff, so its cap follows the pane instead of a fixed size.
    static func expanded(paneHeight: CGFloat) -> CGFloat {
        max(collapsed * 2, paneHeight * 0.3)
    }
}
