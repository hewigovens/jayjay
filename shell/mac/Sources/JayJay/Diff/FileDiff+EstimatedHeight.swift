import CoreGraphics
import JayJayCore

extension FileDiff {
    func estimatedCardHeight(typography: Typography) -> CGFloat {
        let lineHeight = max(18, CGFloat(typography.codeSize) + 5) * typography.codeLineSpacing
        return max(CGFloat(max(lines.count, 1)) * lineHeight + 24, 44)
    }
}
