@testable import JayJay
import JayJayCore
import XCTest

final class ExternalToolInvocationTests: XCTestCase {
    func testCocoaPersistenceArgumentsDoNotTurnRepoLaunchIntoLegacyDiff() {
        XCTAssertNil(ExternalToolInvocation.parse(arguments: [
            "JayJay",
            "-ApplePersistenceIgnoreState", "YES",
            "--repo", "/tmp/repository"
        ]))
    }

    func testCocoaPersistenceArgumentsDoNotHideExternalMergeInvocation() throws {
        let invocation = try XCTUnwrap(ExternalToolInvocation.parse(arguments: [
            "JayJay",
            "-ApplePersistenceIgnoreState", "YES",
            "tool", "merge",
            "/tmp/left.swift", "/tmp/base.swift", "/tmp/right.swift", "/tmp/output.swift",
            "Sources/Conflict.swift", "7"
        ]))

        guard case let .merge(_, _, _, _, path, markerLength, outputIsInitialized) = invocation else {
            return XCTFail("Expected merge invocation")
        }
        XCTAssertEqual(path, "Sources/Conflict.swift")
        XCTAssertEqual(markerLength, 7)
        XCTAssertFalse(outputIsInitialized)
    }
}
