import AppKit
@testable import JayJay
import JayJayCore
import JayJayDiffUI
import XCTest

final class DiffSectionProjectionTests: XCTestCase {
    private var tempDir: URL!

    override func setUpWithError() throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("jayjay-diff-section-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let tempDir {
            try? FileManager.default.removeItem(at: tempDir)
        }
        tempDir = nil
    }

    func testFailedProjectionShowsItsRawFallback() {
        XCTAssertTrue(DiffSection.hasCurrentRenderableDiff(
            loadedPath: "broken.ipynb",
            hunkPath: "broken.ipynb",
            hasRenderedDiff: true,
            loadedProjectionMode: .raw,
            requestedProjectionMode: .processed,
            projectionFailed: true
        ))
    }

    func testRichPreviewStateDoesNotCarryAcrossPaths() {
        let selection = DiffRichPreviewSelection(kind: .projection, path: "analysis.ipynb")

        XCTAssertTrue(selection.isActive(.projection, path: "analysis.ipynb"))
        XCTAssertFalse(selection.isActive(.projection, path: "results.sarif"))
        XCTAssertFalse(selection.isActive(.markdown, path: "analysis.ipynb"))
    }

    func testSameFileProjectionReloadKeepsCurrentDiffVisible() {
        XCTAssertTrue(DiffSection.hasCurrentRenderableDiff(
            loadedPath: "analysis.ipynb",
            hunkPath: "analysis.ipynb",
            hasRenderedDiff: true,
            loadedProjectionMode: .raw,
            requestedProjectionMode: .raw
        ))
        XCTAssertFalse(DiffSection.shouldShowBlockingProgress(isComputing: true, hasCurrentDiff: true))
    }

    func testProjectionModeSwitchDoesNotRenderOldModeDiff() {
        XCTAssertFalse(DiffSection.hasCurrentRenderableDiff(
            loadedPath: "results.sarif",
            hunkPath: "results.sarif",
            hasRenderedDiff: true,
            loadedProjectionMode: .processed,
            requestedProjectionMode: .raw
        ))
    }

    func testHTMLExternalOpenDoesNotRequireWorkingCopySelection() throws {
        let file = tempDir.appendingPathComponent("docs/guide.html")
        try FileManager.default.createDirectory(
            at: file.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try "<h1>Guide</h1>".write(to: file, atomically: true, encoding: .utf8)

        XCTAssertTrue(DiffSection.canOpenHTMLExternally(
            isHTMLFile: true,
            hasProjection: false,
            repoPath: tempDir.path,
            path: "docs/guide.html"
        ))
        XCTAssertFalse(DiffSection.canOpenHTMLExternally(
            isHTMLFile: true,
            hasProjection: true,
            repoPath: tempDir.path,
            path: "docs/guide.html"
        ))
        XCTAssertFalse(DiffSection.canOpenHTMLExternally(
            isHTMLFile: true,
            hasProjection: false,
            repoPath: tempDir.path,
            path: "missing.html"
        ))
    }

    func testHTMLPreviewScriptHintDetectsScriptTagsCaseInsensitively() {
        XCTAssertTrue(DiffSection.htmlContentMayContainScript("<p>hi</p><SCRIPT>alert(1)</SCRIPT>"))
        XCTAssertTrue(DiffSection.htmlContentMayContainScript("<script src=\"app.js\"></script>"))
        XCTAssertFalse(DiffSection.htmlContentMayContainScript("<p>no scripting here</p>"))
        XCTAssertFalse(DiffSection.htmlContentMayContainScript(nil))
    }

    func testInitialProjectionLoadShowsBlockingProgress() {
        XCTAssertFalse(DiffSection.hasCurrentRenderableDiff(
            loadedPath: nil,
            hunkPath: "analysis.ipynb",
            hasRenderedDiff: false,
            loadedProjectionMode: nil,
            requestedProjectionMode: .raw
        ))
        XCTAssertTrue(DiffSection.shouldShowBlockingProgress(isComputing: true, hasCurrentDiff: false))
    }

    func testRichPreviewGutterUsesStablePreviewWidth() {
        let font = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)
        let line = DiffLine(
            oldLineNo: nil,
            newLineNo: 1024,
            style: .added,
            spans: [DiffSpan(text: "value", style: .added, token: .plain)],
            conflictKind: .none,
            noEofNewline: false,
            contextRegion: nil
        )

        let width = DiffSection.richPreviewGutterWidth(
            font: font,
            showsNoteColumn: false,
            hasVisibleNoteMarker: false
        )

        XCTAssertEqual(
            width,
            DiffGutterMetrics.richPreviewWidth(font: font)
        )
        XCTAssertLessThan(width, DiffGutterMetrics.unifiedWidth(displayLines: [line], font: font))
    }

    func testProjectedReviewableDiffsReserveReviewNoteGutterColumn() {
        XCTAssertTrue(DiffSection.reservesReviewNoteGutterColumn(
            hasReviewStore: true,
            reviewChangeId: "change",
            reviewIdentity: "identity",
            compareFromRev: nil
        ))
        XCTAssertFalse(DiffSection.reservesReviewNoteGutterColumn(
            hasReviewStore: true,
            reviewChangeId: "change",
            reviewIdentity: "identity",
            compareFromRev: "other"
        ))
        XCTAssertFalse(DiffSection.reservesReviewNoteGutterColumn(
            hasReviewStore: true,
            reviewChangeId: nil,
            reviewIdentity: "identity",
            compareFromRev: nil
        ))
    }
}
