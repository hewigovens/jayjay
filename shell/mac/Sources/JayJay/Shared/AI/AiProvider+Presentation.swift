import JayJayCore
import SwiftUI

extension AiProvider {
    var label: String {
        aiProviderLabel(provider: self)
    }

    var icon: String {
        switch self {
            case .appleIntelligence: "apple.intelligence"
            case .codex: "chevron.left.forwardslash.chevron.right"
            case .claude: "asterisk"
        }
    }

    /// `</>` is wider than the 16pt settings icon column.
    var iconScale: Image.Scale {
        self == .codex ? .small : .medium
    }
}
