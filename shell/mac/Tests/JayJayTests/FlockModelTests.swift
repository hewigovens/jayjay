import Foundation
@testable import JayJay
import JayJayCore
import XCTest

@MainActor
final class FlockModelTests: XCTestCase {
    func testPerkAccessDoesNotCreateVerifiedMembership() throws {
        let suite = "FlockModelTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        let flock = FlockModel(defaults: defaults)

        flock.choose(.fishOwl)
        XCTAssertNil(flock.member)
        #if DEBUG
            XCTAssertTrue(flock.isUnlocked)
            XCTAssertEqual(flock.activeIcon, .fishOwl)
        #else
            XCTAssertFalse(flock.isUnlocked)
            XCTAssertEqual(flock.activeIcon, .classic)
        #endif

        let member = FlockMember(id: "sponsor", login: "supporter", name: "First Supporter")
        flock.accept(member)
        XCTAssertTrue(flock.isUnlocked)
        XCTAssertEqual(flock.member, member)
        flock.choose(.goldenPheasant)
        XCTAssertEqual(flock.activeIcon, .goldenPheasant)
    }
}
