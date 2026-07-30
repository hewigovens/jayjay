@testable import JayJay
import JayJayCore
import JayJayDiffUI
import XCTest

final class DiffSectionContextExpansionTests: XCTestCase {
    func testExpansionStateQueuesLatestRequestAndResetInvalidatesAttempt() throws {
        var state = DiffContextExpansionState()
        let first = DiffContextExpansionRequest(regionId: 1, action: .showMore(lineCount: 10))
        let latest = DiffContextExpansionRequest(regionId: 2, action: .showAll)
        let attempt = try XCTUnwrap(state.start(first))

        XCTAssertNil(state.start(latest))
        XCTAssertEqual(state.pendingRequest, latest)

        state.reset()

        XCTAssertGreaterThan(state.generation, attempt.generation)
        XCTAssertFalse(state.isInFlight)
        XCTAssertNil(state.pendingRequest)
        XCTAssertNotNil(state.start(first))
    }

    func testAcceptsOnlyLatestExpansionForCurrentDiffBasis() {
        let first = DiffContextExpansionIdentity(
            compareFromRev: nil,
            commitId: "commit-a",
            rev: "change-a",
            path: "file.swift",
            ignoreWhitespace: false,
            projectionMode: "raw"
        )
        let second = DiffContextExpansionIdentity(
            compareFromRev: nil,
            commitId: "commit-b",
            rev: "change-a",
            path: "file.swift",
            ignoreWhitespace: false,
            projectionMode: "raw"
        )
        XCTAssertTrue(DiffSection.shouldAcceptContextExpansion(
            requestIdentity: first,
            currentIdentity: first,
            requestGeneration: 4,
            currentGeneration: 4
        ))
        XCTAssertFalse(DiffSection.shouldAcceptContextExpansion(
            requestIdentity: first,
            currentIdentity: second,
            requestGeneration: 4,
            currentGeneration: 4
        ))
        XCTAssertFalse(DiffSection.shouldAcceptContextExpansion(
            requestIdentity: first,
            currentIdentity: first,
            requestGeneration: 3,
            currentGeneration: 4
        ))
        var whitespaceToggled = first
        whitespaceToggled = DiffContextExpansionIdentity(
            compareFromRev: first.compareFromRev,
            commitId: first.commitId,
            rev: first.rev,
            path: first.path,
            ignoreWhitespace: true,
            projectionMode: first.projectionMode
        )
        XCTAssertFalse(DiffSection.shouldAcceptContextExpansion(
            requestIdentity: first,
            currentIdentity: whitespaceToggled,
            requestGeneration: 4,
            currentGeneration: 4
        ))
        XCTAssertFalse(DiffSection.shouldAcceptContextExpansion(
            requestIdentity: first,
            currentIdentity: nil,
            requestGeneration: 4,
            currentGeneration: 4
        ))
    }

    func testQueuedExpandAllSurvivesItsConsumedRegionId() {
        let region = ContextRegion(
            id: 9,
            oldStartLine: 5,
            newStartLine: 5,
            lineCount: 12,
            initialLineCount: 12
        )
        let separator = DiffLine(
            oldLineNo: nil,
            newLineNo: nil,
            style: .separator,
            spans: [],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: region
        )
        let diff = FileDiff(
            path: "file.swift",
            language: "swift",
            lines: [separator],
            whitespaceOnlyHidden: false
        )
        let staleId = DiffContextExpansionRequest(regionId: 999, action: .showAllRegions)
        XCTAssertTrue(DiffSection.requestTargetsAvailableRegion(staleId, in: diff))
        XCTAssertFalse(DiffSection.requestTargetsAvailableRegion(
            DiffContextExpansionRequest(regionId: 999, action: .showAll),
            in: diff
        ))
        XCTAssertFalse(DiffSection.requestTargetsAvailableRegion(
            staleId,
            in: FileDiff(path: "file.swift", language: "swift", lines: [], whitespaceOnlyHidden: false)
        ))
    }
}
