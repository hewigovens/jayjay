@testable import JayJay
import XCTest

@MainActor
final class SettingsSnapshotTests: XCTestCase {
    func testBlockingReadLeavesMainActorAvailableAndReusesLoadedValue() async {
        let snapshot = SettingsSnapshot<Int>()
        let started = expectation(description: "background read started")
        let release = DispatchSemaphore(value: 0)
        let load = Task {
            await snapshot.load {
                XCTAssertFalse(Thread.isMainThread)
                started.fulfill()
                XCTAssertEqual(release.wait(timeout: .now() + 5), .success)
                return 42
            }
        }
        await fulfillment(of: [started], timeout: 2)
        XCTAssertNil(snapshot.value)
        release.signal()
        await load.value
        XCTAssertEqual(snapshot.value, 42)

        await snapshot.load {
            XCTFail("a loaded tab must not rerun its probe")
            return -1
        }
        XCTAssertEqual(snapshot.value, 42)
    }

    func testCancelledTabReusesItsPendingReadWhenReopened() async {
        let snapshot = SettingsSnapshot<Int>()
        let started = expectation(description: "background read started")
        let release = DispatchSemaphore(value: 0)
        let load = Task {
            await snapshot.load {
                started.fulfill()
                XCTAssertEqual(release.wait(timeout: .now() + 5), .success)
                return 42
            }
        }
        await fulfillment(of: [started], timeout: 2)
        load.cancel()
        release.signal()
        await load.value
        XCTAssertNil(snapshot.value, "a cancelled view task must not publish its result")

        await snapshot.load {
            XCTFail("reopening must reuse the pending probe")
            return -1
        }
        XCTAssertEqual(snapshot.value, 42)
    }
}
