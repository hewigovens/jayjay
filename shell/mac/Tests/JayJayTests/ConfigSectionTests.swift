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
        // The flattened listing is not grouped, so the same section can reappear after others. Duplicate section ids make SwiftUI Form reuse the wrong cells (empty ui group, headers inside another section).
        let sections = ConfigSection.parse(
            "operation.hostname = host\nui.editor = code\nuser.name = Alice\nui.diff = split\n"
        )

        XCTAssertEqual(sections.map(\.name), ["operation", "ui", "user"])
        XCTAssertEqual(sections[1].entries.map(\.key), ["editor", "diff"])
        XCTAssertEqual(Set(sections.map(\.id)).count, sections.count)
        let entryIds = sections.flatMap { $0.entries.map(\.id) }
        XCTAssertEqual(Set(entryIds).count, entryIds.count)
    }

    func testParseKeepsEqualsInsideKey() {
        let sections = ConfigSection.parse(
            "remotes.foo=bar.auto-track-bookmarks = \"glob:*\"\n"
        )

        XCTAssertEqual(sections.map(\.name), ["remotes"])
        XCTAssertEqual(sections[0].entries.map(\.key), ["foo=bar.auto-track-bookmarks"])
        XCTAssertEqual(sections[0].entries[0].value, "\"glob:*\"")
    }

    func testEmptyPathAndListingIsMissingConfig() {
        XCTAssertTrue(JjConfigSnapshot(path: "", sections: []).isMissing)
        XCTAssertFalse(JjConfigSnapshot(path: "/tmp/jj/config.toml", sections: []).isMissing)
        XCTAssertFalse(
            JjConfigSnapshot(path: "", sections: ConfigSection.parse("user.name = Alice")).isMissing
        )
        XCTAssertFalse(
            JjConfigSnapshot(path: "/tmp/jj/config.toml", sections: [], error: "bad.toml: invalid").isMissing
        )
    }
}
