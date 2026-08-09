@testable import JayJay
import JayJayCore
import XCTest

final class FileEditorGateTests: XCTestCase {
    func testWorkingCopyTextFileCanBeEdited() {
        XCTAssertTrue(
            ChangeDetailView.canEditWorkingCopyFile(
                info: mockChangeInfo(isWorkingCopy: true),
                isCompareMode: false,
                hunk: testHunk(path: "main.swift", oldContent: "let old = true\n", newContent: "let new = true\n", hunkType: .modified),
                hasConflict: false
            )
        )
    }

    func testHistoricalAndConflictedFilesCannotBeEdited() {
        XCTAssertFalse(
            ChangeDetailView.canEditWorkingCopyFile(
                info: mockChangeInfo(),
                isCompareMode: false,
                hunk: testHunk(path: "main.swift", hunkType: .modified),
                hasConflict: false
            )
        )
        XCTAssertFalse(
            ChangeDetailView.canEditWorkingCopyFile(
                info: mockChangeInfo(isWorkingCopy: true),
                isCompareMode: false,
                hunk: testHunk(path: "main.swift", hunkType: .modified),
                hasConflict: true
            )
        )
    }
}
