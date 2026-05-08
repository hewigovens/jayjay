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

    func testPreservesEmptyQuotedJjArguments() {
        XCTAssertEqual(
            parseJjCommandArgs(command: #"describe -m """#),
            ["describe", "-m", ""]
        )
    }

    func testRejectsUnclosedQuote() {
        XCTAssertNil(parseJjCommandArgs(command: #"log -r "mine()"#))
    }

    func testHistoryDedupesAndKeepsNewestFirst() {
        let defaults = makeDefaults()

        _ = CommandPaletteHistory.record("status", defaults: defaults)
        _ = CommandPaletteHistory.record("log -r @", defaults: defaults)
        let history = CommandPaletteHistory.record("status", defaults: defaults)

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

    private func makeDefaults() -> UserDefaults {
        let suiteName = "dev.hewig.jayjay.palette.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }
}
