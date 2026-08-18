@testable import JayJay
import XCTest

final class ConfigSectionTests: XCTestCase {
    func testParseGroupsByPrefix() {
        let sections = ConfigSection.parse(
            "user.name = Alice\nuser.email = a@example.com\nui.diff = split\n"
        )

        XCTAssertEqual(sections.map(\.name), ["user", "ui"])
        XCTAssertEqual(sections[0].entries.map(\.key), ["name", "email"])
        XCTAssertEqual(sections[0].entries[1].value, "a@example.com")
        XCTAssertEqual(sections[1].entries[0].value, "split")
    }

    func testParseMergesNonContiguousOccurrencesIntoUniqueIdentities() {
        // `jj config list` is not grouped, so the same section can reappear after others. Duplicate section ids make SwiftUI Form reuse the wrong cells (empty ui group, headers inside another section).
        let sections = ConfigSection.parse(
            "operation.hostname = host\nui.editor = code\nuser.name = Alice\nui.diff = split\n"
        )

        XCTAssertEqual(sections.map(\.name), ["operation", "ui", "user"])
        XCTAssertEqual(sections[1].entries.map(\.key), ["editor", "diff"])
        XCTAssertEqual(Set(sections.map(\.id)).count, sections.count)
        let entryIds = sections.flatMap { $0.entries.map(\.id) }
        XCTAssertEqual(Set(entryIds).count, entryIds.count)
    }
}
