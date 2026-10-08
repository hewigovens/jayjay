@testable import JayJay
import XCTest

@MainActor
final class FSEventSubscriptionTests: XCTestCase {
    private func makeDirectory() throws -> URL {
        let directory = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-fsevents-\(UUID().uuidString)")
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        addTeardownBlock { try? FileManager.default.removeItem(at: directory) }
        return directory
    }

    func testDeliversEventsUnderThePath() throws {
        let directory = try makeDirectory()
        let delivered = expectation(description: "file event delivered")
        delivered.assertForOverFulfill = false
        let subscription = FSEventSubscription(path: directory.path, latency: 0.1) { paths in
            if paths.contains(where: { $0.hasSuffix("touched.txt") }) {
                delivered.fulfill()
            }
        }
        XCTAssertNotNil(subscription)

        try Data().write(to: directory.appending(path: "touched.txt"))

        wait(for: [delivered], timeout: 5)
        withExtendedLifetime(subscription) {}
    }

    func testDroppingTheSubscriptionReleasesItsHandler() throws {
        let directory = try makeDirectory()
        var sentinel: Sentinel? = Sentinel()
        weak let released = sentinel
        var subscription = FSEventSubscription(path: directory.path, latency: 0.1) { [sentinel] _ in
            _ = sentinel
        }
        XCTAssertNotNil(subscription)
        sentinel = nil
        XCTAssertNotNil(released, "the stream must keep its handler alive")

        subscription = nil
        _ = subscription

        let freed = expectation(for: NSPredicate { _, _ in released == nil }, evaluatedWith: nil)
        wait(for: [freed], timeout: 5)
    }
}

private final class Sentinel: Sendable {}
