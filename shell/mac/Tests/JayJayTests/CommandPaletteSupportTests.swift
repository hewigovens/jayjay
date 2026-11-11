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

    func testCommandPaletteSearchNormalizesHelpQueries() {
        let items = [
            CommandPaletteItem(
                title: "Split work from a change",
                icon: "questionmark.circle",
                category: "Help",
                detail: "Select hunks or line ranges and extract them into another change.",
                keywords: ["help", "split", "diff edit", "hunk"],
                isHelpTopic: true
            ) {},
            CommandPaletteItem(title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View") {}
        ]

        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "help split", items: items).map(\.title),
            ["Split work from a change"]
        )
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "how do i split a change", items: items).map(\.title),
            ["Split work from a change"]
        )
    }

    func testBundledHelpFeaturesAreSearchable() {
        let helpItems = HelpFeatureIndex.bundled.map { feature in
            CommandPaletteItem(
                title: feature.title,
                icon: "questionmark.circle",
                category: "Help",
                detail: feature.summary,
                keywords: feature.commandPaletteKeywords,
                shortcut: feature.shortcut,
                isHelpTopic: true
            ) {}
        }

        XCTAssertGreaterThan(helpItems.count, 5)
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "help stacked pr", items: helpItems).first?.title,
            "Create stacked pull requests"
        )
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "? compare two changes", items: helpItems).first?.title,
            "Compare two changes"
        )
    }

    func testHelpTopicsFollowEveryOtherMatch() {
        let items = [
            CommandPaletteItem(
                title: "Operate on changes",
                icon: "questionmark.circle",
                category: "Help",
                detail: "Describe, squash, abandon, duplicate, or undo changes.",
                keywords: ["help", "duplicate"],
                isHelpTopic: true
            ) {},
            CommandPaletteItem(title: "Duplicate", icon: "plus.square.on.square", category: "Change") {}
        ]

        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "duplicate", items: items).map(\.title),
            ["Duplicate", "Operate on changes"]
        )
        XCTAssertEqual(
            CommandPaletteSearch.rank(query: "squash", items: items).map(\.title),
            ["Operate on changes"]
        )
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
