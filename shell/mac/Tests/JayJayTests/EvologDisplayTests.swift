@testable import JayJay
import XCTest

final class EvologDisplayTests: XCTestCase {
    func testOperationLabelMapsKnownOperationsInUiLayer() {
        XCTAssertEqual(EvologDisplay.operationLabel("snapshot working copy 123"), "snapshot")
        XCTAssertEqual(EvologDisplay.operationLabel("describe commit abc"), "describe")
        XCTAssertEqual(EvologDisplay.operationLabel("rebase commit abc"), "rebase")
        XCTAssertEqual(EvologDisplay.operationLabel("squash commits abc"), "squash")
        XCTAssertEqual(EvologDisplay.operationLabel("split commit abc"), "split")
        XCTAssertEqual(EvologDisplay.operationLabel("new empty commit"), "new")
        XCTAssertEqual(EvologDisplay.operationLabel(""), "rewrite")
    }

    func testOperationLabelPreservesUnknownOperation() {
        XCTAssertEqual(EvologDisplay.operationLabel("custom operation"), "custom operation")
    }
}
