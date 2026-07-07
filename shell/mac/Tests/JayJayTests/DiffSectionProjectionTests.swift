@testable import JayJay
import JayJayCore
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

    func testPlistProjectionUsesProcessedModeWithoutRichToggle() {
        let projection = testProjection(
            pluginId: "plist",
            pluginLabel: "Property list",
            mode: .raw,
            renderKind: .text,
            virtualPath: "Info.plist.xml"
        )

        XCTAssertTrue(DiffProjectionDisplayPolicy.opensAutomatically(projection))
        XCTAssertEqual(DiffProjectionDisplayPolicy.requestMode(for: projection, richView: false), .processed)
    }

    func testNotebookProjectionStillUsesRichToggle() {
        let projection = testProjection(pluginId: "ipynb", pluginLabel: "Notebook", mode: .raw)

        XCTAssertFalse(DiffProjectionDisplayPolicy.opensAutomatically(projection))
        XCTAssertEqual(DiffProjectionDisplayPolicy.requestMode(for: projection, richView: false), .raw)
        XCTAssertEqual(DiffProjectionDisplayPolicy.requestMode(for: projection, richView: true), .processed)
    }

    func testSarifProjectionStillUsesRichToggle() {
        let projection = testProjection(
            pluginId: "sarif",
            pluginLabel: "SARIF",
            mode: .raw,
            renderKind: .markdown
        )

        XCTAssertFalse(DiffProjectionDisplayPolicy.opensAutomatically(projection))
        XCTAssertEqual(DiffProjectionDisplayPolicy.requestMode(for: projection, richView: false), .raw)
        XCTAssertEqual(DiffProjectionDisplayPolicy.requestMode(for: projection, richView: true), .processed)
    }

    func testRichPreviewStateDoesNotCarryAcrossPaths() {
        let selection = DiffRichPreviewSelection(kind: .projection, path: "analysis.ipynb")

        XCTAssertTrue(selection.isActive(.projection, path: "analysis.ipynb"))
        XCTAssertFalse(selection.isActive(.projection, path: "results.sarif"))
        XCTAssertFalse(selection.isActive(.markdown, path: "analysis.ipynb"))
    }

    func testPlistBannerExplainsBinaryPreview() {
        let projection = testProjection(
            pluginId: "plist",
            pluginLabel: "Property list",
            mode: .processed,
            renderKind: .text,
            virtualPath: "Info.plist.xml"
        )

        XCTAssertTrue(DiffProjectionDisplayPolicy.showsBanner(for: projection, richView: false))
        XCTAssertEqual(
            DiffProjectionDisplayPolicy.title(for: projection),
            "Binary property list on disk, previewed as XML"
        )
    }

    func testSameFileProjectionReloadKeepsCurrentDiffVisible() {
        XCTAssertTrue(DiffSection.shouldKeepLoadedContentWhileLoading(
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
        XCTAssertFalse(DiffSection.shouldKeepLoadedContentWhileLoading(
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

    func testInitialProjectionLoadShowsBlockingProgress() {
        XCTAssertFalse(DiffSection.shouldKeepLoadedContentWhileLoading(
            loadedPath: nil,
            hunkPath: "analysis.ipynb",
            hasRenderedDiff: false,
            loadedProjectionMode: nil,
            requestedProjectionMode: .raw
        ))
        XCTAssertTrue(DiffSection.shouldShowBlockingProgress(isComputing: true, hasCurrentDiff: false))
    }
}
