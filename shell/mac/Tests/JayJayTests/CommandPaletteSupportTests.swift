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

    func testHistoryDedupesAndKeepsNewestFirst() {
        var history: [String] = []

        history = CommandPaletteHistory.record("status", in: history)
        history = CommandPaletteHistory.record("log -r @", in: history)
        history = CommandPaletteHistory.record("status", in: history)

        XCTAssertEqual(Array(history.prefix(2)), ["status", "log -r @"])
    }

    func testHistoryRecallWalksOlderAndNewerEntries() {
        let history = ["status", "log -r @", "diff --stat"]

        let first = CommandPaletteHistory.recall(history: history, historyIndex: nil, older: true)
        XCTAssertEqual(first?.query, "jj status")
        XCTAssertEqual(first?.historyIndex, 0)

        let second = CommandPaletteHistory.recall(
            history: history,
            historyIndex: first?.historyIndex,
            older: true
        )
        XCTAssertEqual(second?.query, "jj log -r @")
        XCTAssertEqual(second?.historyIndex, 1)

        let newer = CommandPaletteHistory.recall(
            history: history,
            historyIndex: second?.historyIndex,
            older: false
        )
        XCTAssertEqual(newer?.query, "jj status")
        XCTAssertEqual(newer?.historyIndex, 0)

        let liveQuery = CommandPaletteHistory.recall(
            history: history,
            historyIndex: newer?.historyIndex,
            older: false
        )
        XCTAssertEqual(liveQuery?.query, "jj ")
        XCTAssertNil(liveQuery?.historyIndex)
    }

    func testCommandPaletteSearchMatchesKeywords() {
        let item = CommandPaletteItem(
            title: "Toggle Tree File List",
            icon: "list.bullet.indent",
            category: "View",
            keywords: ["tree", "file", "folder", "list"]
        ) {}

        XCTAssertTrue(item.matches(query: "tree"))
        XCTAssertTrue(item.matches(query: "folder"))
        XCTAssertTrue(item.matches(query: "tree list"))
        XCTAssertTrue(item.matches(query: "toggle tr"))
        XCTAssertFalse(item.matches(query: "bookmark"))

        let refresh = CommandPaletteItem(title: "Refresh", icon: "arrow.triangle.2.circlepath", category: "View") {}
        XCTAssertFalse(refresh.matches(query: "toggle tr"))
    }
}
