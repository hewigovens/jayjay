import JayJayCore
import JayJayDiffUI
import SwiftUI

extension DiffEditFileSection {
    var currentSelectedLineRange: ClosedRange<Int>? {
        nil
    }

    func didSelectLines(_ lineRange: ClosedRange<Int>) {}

    func selectFile() {
        onSelectFile()
    }

    func selectChangeGroup(_ lineRange: ClosedRange<Int>) {
        let mapped = ClosedRange(
            uncheckedBounds: (
                displayToFullMap[lineRange.lowerBound] ?? lineRange.lowerBound,
                displayToFullMap[lineRange.upperBound] ?? lineRange.upperBound
            )
        )
        onSelectHunk(mapped)
    }

    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState? {
        guard let fileDiff,
              let fullLine = displayToFullMap[lineNumber],
              fileDiff.lines.indices.contains(fullLine - 1),
              fileDiff.lines[fullLine - 1].isChanged
        else { return nil }
        return selectedChangedLines.contains(fullLine) ? .selected : .unselected
    }

    func toggleLineCheckbox(_ lineNumber: Int) {
        guard let fullLine = displayToFullMap[lineNumber] else { return }
        onToggleLine(fullLine)
    }
}
