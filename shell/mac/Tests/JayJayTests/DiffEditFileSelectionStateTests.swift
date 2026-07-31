@testable import JayJay
import Observation
import XCTest

@MainActor
final class DiffEditFileSelectionStateTests: XCTestCase {
    func testNoOpAndOtherFileMutationDoNotInvalidateObserver() {
        let first = DiffEditFileSelectionState()
        let second = DiffEditFileSelectionState()
        let firstInvalidated = expectation(description: "First file selection invalidated")
        firstInvalidated.isInverted = true

        withObservationTracking {
            _ = first.selectedChangedLines
        } onChange: {
            firstInvalidated.fulfill()
        }

        first.replace(with: [])
        second.replace(with: [7])
        wait(for: [firstInvalidated], timeout: 0.01)
    }

    func testMutationInvalidatesObserver() {
        let selection = DiffEditFileSelectionState()
        let invalidated = expectation(description: "Selection invalidated")

        withObservationTracking {
            _ = selection.selectedChangedLines
        } onChange: {
            invalidated.fulfill()
        }

        selection.replace(with: [3])
        wait(for: [invalidated], timeout: 0.01)
    }
}
