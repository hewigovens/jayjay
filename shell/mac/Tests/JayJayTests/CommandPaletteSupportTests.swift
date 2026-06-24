@testable import JayJay
import JayJayCore
import XCTest

final class CommandPaletteSupportTests: XCTestCase {
    func testParsesQuotedJjArguments() {
        XCTAssertEqual(
            parseJjCommandArgs(command: #"log -r "description(exact:'fix bug')" --limit 5"#),
            ["log", "-r", "description(exact:'fix bug')", "--limit", "5"]
        )
    }

    func testRejectsUnclosedQuote() {
        XCTAssertNil(parseJjCommandArgs(command: #"log -r "mine()"#))
    }

    func testCommandPaletteSearchRanksFuzzyMatches() {
        let items = [
            CommandPaletteItem(
                title: "Toggle Tree File List",
                icon: "list.bullet.indent",
                category: "View",
                keywords: ["tree", "file", "folder", "list"]
            ) {},
            CommandPaletteItem(title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View") {}
        ]

        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "", items: items).map(\.title),
            ["Toggle Tree File List", "Refresh"]
        )
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "toggle tr", items: items).map(\.title),
            ["Toggle Tree File List"]
        )
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "ttfl", items: items).map(\.title),
            ["Toggle Tree File List"]
        )
        XCTAssertTrue(CommandPaletteSearch.rank(query: "bookmark", items: items).isEmpty)
    }

    func testKeybindRowsAreInfoOnlyAndShortcutsAreSearchable() {
        let items = [
            CommandPaletteItem(
                title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View", shortcut: "⌘R"
            ) {},
            CommandPaletteItem.keybind(
                title: "Mark File Reviewed", icon: "checkmark.circle", shortcut: "Space",
                keywords: ["review", "reviewed", "check", "diff"]
            )
        ]

        // Keybind rows are info-only (the row never executes); commands carry an action.
        XCTAssertFalse(items[0].isInfo)
        XCTAssertTrue(items[1].isInfo)

        // The keybind row is findable by its shortcut text, not just its title.
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "space", items: items).map(\.title),
            ["Mark File Reviewed"]
        )
    }
}
