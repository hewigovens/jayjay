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

    func toggleChangeGroup(_ lineRange: ClosedRange<Int>) {
        onToggleGroup(lineRange.compactMap { displayToFullMap[$0] })
    }

    func lineCheckboxState(for lineNumber: Int) -> DiffGutterCheckboxState? {
        guard let fullLine = displayToFullMap[lineNumber], changedLines.contains(fullLine)
        else { return nil }
        return selectedChangedLines.contains(fullLine) ? .selected : .unselected
    }

    func toggleLineCheckbox(_ lineNumber: Int) {
        guard let fullLine = displayToFullMap[lineNumber] else { return }
        onToggleLine(fullLine)
    }
}
