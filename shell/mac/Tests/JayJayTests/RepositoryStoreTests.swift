@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class RepositoryStoreTests: XCTestCase {
    func testStoresPinsThroughSharedRustFile() throws {
        let directory = FileManager.default.temporaryDirectory
            .appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let storePath = directory.appendingPathComponent("repositories.json").path
        let repository = directory.appendingPathComponent("repo", isDirectory: true)
        try FileManager.default.createDirectory(at: repository, withIntermediateDirectories: true)

        let swiftUIStore = RepositoryStore(storePath: storePath)
        swiftUIStore.setPinned(true, path: repository.path)

        let otherShell = RepositoryStore(storePath: storePath)
        let normalizedPath = normalizedRepositoryPath(path: repository.path)
        XCTAssertTrue(otherShell.paths.contains(normalizedPath))

        otherShell.setPinned(false, path: repository.path)
        XCTAssertFalse(swiftUIStore.paths.contains(normalizedPath))
    }
}
