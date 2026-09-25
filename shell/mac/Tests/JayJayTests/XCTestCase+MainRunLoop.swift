import Foundation
import XCTest

extension XCTestCase {
    @MainActor
    func runUntilMainRunLoopWaits(file: StaticString = #filePath, line: UInt = #line) {
        var waited = false
        let observer = CFRunLoopObserverCreateWithHandler(nil, CFRunLoopActivity.beforeWaiting.rawValue, false, 0) { _, _ in
            waited = true
        }
        CFRunLoopAddObserver(CFRunLoopGetMain(), observer, .commonModes)
        defer { CFRunLoopObserverInvalidate(observer) }
        let deadline = Date(timeIntervalSinceNow: 2)
        while !waited, Date() < deadline {
            RunLoop.main.run(mode: .default, before: deadline)
        }
        XCTAssertTrue(waited, "the main run loop never went idle", file: file, line: line)
    }
}
