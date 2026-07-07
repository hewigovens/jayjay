@testable import JayJay
import XCTest

final class RepositoryActionsTests: XCTestCase {
    private var tempDir: URL!

    override func setUpWithError() throws {
        tempDir = FileManager.default.temporaryDirectory
            .appendingPathComponent("jayjay-actions-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: tempDir, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        if let tempDir {
            try? FileManager.default.removeItem(at: tempDir)
        }
        tempDir = nil
    }

    func testFileViewerSelectionURLUsesExistingFile() throws {
        let file = tempDir.appendingPathComponent("Sources/App.swift")
        try FileManager.default.createDirectory(
            at: file.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try "app".write(to: file, atomically: true, encoding: .utf8)

        XCTAssertEqual(
            RepositoryActions.fileViewerSelectionURL(repoPath: tempDir.path, path: "Sources/App.swift").path,
            file.path
        )
    }

    func testFileViewerSelectionURLFallsBackToExistingParent() throws {
        let directory = tempDir.appendingPathComponent("Sources")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)

        XCTAssertEqual(
            RepositoryActions.fileViewerSelectionURL(repoPath: tempDir.path, path: "Sources/Missing.swift").path,
            directory.path
        )
    }

    func testFileViewerSelectionURLDoesNotEscapeRepoRoot() throws {
        let sibling = tempDir.deletingLastPathComponent()
            .appendingPathComponent("\(tempDir.lastPathComponent)-sibling")
        try FileManager.default.createDirectory(at: sibling, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: sibling) }

        XCTAssertEqual(
            RepositoryActions.fileViewerSelectionURL(repoPath: tempDir.path, path: "../\(sibling.lastPathComponent)").path,
            tempDir.path
        )
    }

    func testFileViewerRootURLUsesSelectedFilesParent() {
        let file = tempDir.appendingPathComponent("Sources/App.swift")

        XCTAssertEqual(
            RepositoryActions.fileViewerRootURL(selectionURL: file).path,
            tempDir.appendingPathComponent("Sources").path
        )
    }

    func testRevealInFinderSelectsPathInParentDirectory() {
        let file = tempDir.appendingPathComponent("Sources/App.swift")
        let workspace = RecordingFileRevealingWorkspace(selectResult: true)

        XCTAssertTrue(RepositoryActions.revealInFinder(selectionURL: file, workspace: workspace))
        XCTAssertEqual(workspace.selectedPath, file.path)
        XCTAssertEqual(workspace.rootPath, tempDir.appendingPathComponent("Sources").path)
        XCTAssertTrue(workspace.activatedURLs.isEmpty)
    }

    func testRevealInFinderFallsBackToActivateFileViewer() {
        let file = tempDir.appendingPathComponent("Sources/App.swift")
        let workspace = RecordingFileRevealingWorkspace(selectResult: false)

        XCTAssertFalse(RepositoryActions.revealInFinder(selectionURL: file, workspace: workspace))
        XCTAssertEqual(workspace.selectedPath, file.path)
        XCTAssertEqual(workspace.rootPath, tempDir.appendingPathComponent("Sources").path)
        XCTAssertEqual(workspace.activatedURLs, [file])
    }

    func testExistingRepoFileURLUsesFileInsideRepo() throws {
        let file = tempDir.appendingPathComponent("releases/0.3.6.html")
        try FileManager.default.createDirectory(
            at: file.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try "<h1>Release</h1>".write(to: file, atomically: true, encoding: .utf8)

        XCTAssertEqual(
            RepositoryActions.existingRepoFileURL(repoPath: tempDir.path, path: "releases/0.3.6.html")?.path,
            file.path
        )
    }

    func testExistingRepoFileURLRejectsMissingDirectoryAndEscapingPath() throws {
        let directory = tempDir.appendingPathComponent("docs")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let sibling = tempDir.deletingLastPathComponent()
            .appendingPathComponent("\(tempDir.lastPathComponent)-sibling.html")
        try "<h1>Outside</h1>".write(to: sibling, atomically: true, encoding: .utf8)
        defer { try? FileManager.default.removeItem(at: sibling) }

        XCTAssertNil(RepositoryActions.existingRepoFileURL(repoPath: tempDir.path, path: "docs"))
        XCTAssertNil(RepositoryActions.existingRepoFileURL(repoPath: tempDir.path, path: "missing.html"))
        XCTAssertNil(RepositoryActions.existingRepoFileURL(repoPath: tempDir.path, path: "../\(sibling.lastPathComponent)"))
    }

    func testOpenInDefaultAppUsesExistingRepoFile() throws {
        let file = tempDir.appendingPathComponent("release.html")
        try "<h1>Release</h1>".write(to: file, atomically: true, encoding: .utf8)
        let workspace = RecordingFileOpeningWorkspace(openResult: true)

        XCTAssertTrue(
            RepositoryActions.openInDefaultApp(
                repoPath: tempDir.path,
                path: "release.html",
                workspace: workspace
            )
        )
        XCTAssertEqual(workspace.openedURLs, [file])
    }

    func testOpenInDefaultAppSkipsMissingRepoFile() {
        let workspace = RecordingFileOpeningWorkspace(openResult: true)

        XCTAssertFalse(
            RepositoryActions.openInDefaultApp(
                repoPath: tempDir.path,
                path: "missing.html",
                workspace: workspace
            )
        )
        XCTAssertTrue(workspace.openedURLs.isEmpty)
    }
}

private final class RecordingFileRevealingWorkspace: FileRevealingWorkspace {
    let selectResult: Bool
    var selectedPath: String?
    var rootPath: String?
    var activatedURLs: [URL] = []

    init(selectResult: Bool) {
        self.selectResult = selectResult
    }

    func selectFile(_ fullPath: String?, inFileViewerRootedAtPath rootFullPath: String) -> Bool {
        selectedPath = fullPath
        rootPath = rootFullPath
        return selectResult
    }

    func activateFileViewerSelecting(_ fileURLs: [URL]) {
        activatedURLs.append(contentsOf: fileURLs)
    }
}

private final class RecordingFileOpeningWorkspace: FileOpeningWorkspace {
    let openResult: Bool
    var openedURLs: [URL] = []

    init(openResult: Bool) {
        self.openResult = openResult
    }

    func open(_ url: URL) -> Bool {
        openedURLs.append(url)
        return openResult
    }
}
