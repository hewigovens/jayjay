@testable import JayJay
import XCTest

final class RepoListGroupingTests: XCTestCase {
    private let resolutions: [String: RepoPathResolution] = [
        "/work/main": .init(canonicalPath: "/work/main", primaryRoot: "/work/main"),
        "/work/agent-b": .init(canonicalPath: "/work/agent-b", primaryRoot: "/work/main"),
        "/work/agent-a": .init(canonicalPath: "/work/agent-a", primaryRoot: "/work/main"),
        "/work/orphan-ws": .init(canonicalPath: "/work/orphan-ws", primaryRoot: "/gone/main"),
        "/work/other": .init(canonicalPath: "/work/other", primaryRoot: "/work/other")
    ]

    func testRecentWorkspacesNestUnderTheirListedRootSortedByName() {
        let result = RepoListGrouping.groups(
            pinned: ["/work/main"],
            recents: ["/work/agent-b", "/work/other", "/work/agent-a"],
            resolutions: resolutions
        )
        XCTAssertEqual(result.pinned.map(\.path), ["/work/main"])
        XCTAssertEqual(result.pinned[0].workspaces, ["/work/agent-a", "/work/agent-b"])
        XCTAssertEqual(result.recent.map(\.path), ["/work/other"])
    }

    func testWorkspacesWithoutAListedRootStayFlat() {
        let result = RepoListGrouping.groups(
            pinned: [],
            recents: ["/work/orphan-ws", "/not-a-repo"],
            resolutions: resolutions
        )
        XCTAssertEqual(result.recent.map(\.path), ["/work/orphan-ws", "/not-a-repo"])
        XCTAssertTrue(result.recent.allSatisfy(\.workspaces.isEmpty))
    }

    func testPinnedWorkspaceStaysTopLevel() {
        let result = RepoListGrouping.groups(
            pinned: ["/work/agent-a"],
            recents: ["/work/main", "/work/agent-b"],
            resolutions: resolutions
        )
        XCTAssertEqual(result.pinned.map(\.path), ["/work/agent-a"])
        XCTAssertEqual(result.recent.map(\.path), ["/work/main"])
        XCTAssertEqual(result.recent[0].workspaces, ["/work/agent-b"])
    }

    /// Lookups resolve asynchronously, so entries render flat until their resolutions arrive.
    func testUnresolvedPathsRenderFlat() {
        let result = RepoListGrouping.groups(
            pinned: ["/work/main"],
            recents: ["/work/agent-a"],
            resolutions: [:]
        )
        XCTAssertEqual(result.pinned.map(\.path), ["/work/main"])
        XCTAssertEqual(result.recent.map(\.path), ["/work/agent-a"])
        XCTAssertTrue(result.pinned[0].workspaces.isEmpty)
    }
}
