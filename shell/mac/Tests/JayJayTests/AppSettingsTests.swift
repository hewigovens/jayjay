import Foundation
@testable import JayJay
import XCTest

final class AppSettingsTests: XCTestCase {
    func testAnonymousStatsAreDisabledByDefault() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)

        XCTAssertFalse(settings.sendsAnonymousStats)
    }
}
