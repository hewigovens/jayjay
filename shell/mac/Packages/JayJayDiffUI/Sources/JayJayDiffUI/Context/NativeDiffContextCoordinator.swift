import AppKit
import JayJayCore

public final class NativeDiffContextCoordinator: NSObject, NSTextViewDelegate {
    var onExpandContext: ((DiffContextExpansionRequest) -> Void)?
    var selectionRenderCache: SelectionRenderCache?

    struct SelectionRenderCache {
        struct Identity: Equatable {
            let contentGeneration: UInt64
            let reserveNoteColumn: Bool
            let compactGutterWidth: Bool
            let enablesContextExpansion: Bool
            let resetSelectionGeneration: UInt64
            let revealFeedback: DiffContextRevealFeedback?
            let isDark: Bool
            let fontSize: Double
            let fontFamily: String
            let reduceMotion: Bool
            let fitsContent: Bool
            let currentSelectedLineRange: ClosedRange<Int>?
        }

        let identity: Identity
        let gutterContext: NativeDiffGutterRenderContext
        let groupsByIndex: [UInt32: ChangeGroup]
    }

    public func textView(
        _ textView: NSTextView,
        clickedOnLink link: Any,
        at charIndex: Int
    ) -> Bool {
        guard let request = DiffContextExpansionLink.request(from: link),
              let onExpandContext
        else { return false }
        onExpandContext(request)
        return true
    }
}
