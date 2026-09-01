@testable import JayJay
import XCTest

final class CommandLineInterfaceTests: XCTestCase {
    func testGuiLaunchArgumentsFallThrough() {
        XCTAssertNil(CommandLineInterface.outcome(for: ["JayJay"]))
        XCTAssertNil(CommandLineInterface.outcome(for: ["JayJay", "/tmp/some-repo"]))
    }

    func testVersionOutcomeComesFromTheCoreDispatcher() {
        let outcome = CommandLineInterface.outcome(for: ["JayJay", "--version"])
        XCTAssertEqual(outcome?.exitCode, 0)
        XCTAssertEqual(outcome?.message, "jayjay \(AppMetadata.shortVersion)\n")
    }
}
