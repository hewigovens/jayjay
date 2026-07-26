@testable import JayJay
import XCTest

final class DiffEditFocusTests: XCTestCase {
    private let paths = ["a", "b", "c"]

    func testForwardFromNilFocusesFirst() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: nil, paths: paths, forward: true), "a")
    }

    func testBackwardFromNilFocusesLast() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: nil, paths: paths, forward: false), "c")
    }

    func testForwardMovesToNext() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "a", paths: paths, forward: true), "b")
    }

    func testBackwardMovesToPrevious() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "c", paths: paths, forward: false), "b")
    }

    func testForwardClampsAtLast() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "c", paths: paths, forward: true), "c")
    }

    func testBackwardClampsAtFirst() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "a", paths: paths, forward: false), "a")
    }

    func testStalePathForwardFocusesFirst() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "gone", paths: paths, forward: true), "a")
    }

    func testStalePathBackwardFocusesLast() {
        XCTAssertEqual(DiffEditSession.nextFocusedPath(current: "gone", paths: paths, forward: false), "c")
    }

    func testEmptyPathsHasNoFocus() {
        XCTAssertNil(DiffEditSession.nextFocusedPath(current: nil, paths: [], forward: true))
        XCTAssertNil(DiffEditSession.nextFocusedPath(current: "a", paths: [], forward: false))
    }
}
