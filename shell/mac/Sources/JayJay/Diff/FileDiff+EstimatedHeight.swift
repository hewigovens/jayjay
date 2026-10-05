import CoreGraphics
import JayJayCore

extension FileDiff {
    func estimatedCardHeight(fontSize: Double) -> CGFloat {
        let lineHeight = max(18, CGFloat(fontSize) + 5)
        return max(CGFloat(max(lines.count, 1)) * lineHeight + 24, 44)
    }
}
