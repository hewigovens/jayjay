import Foundation
@testable import JayJay
import JayJayCore
import XCTest

final class AppSettingsTests: XCTestCase {
    func testAnonymousStatsAreDisabledByDefault() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)

        XCTAssertFalse(settings.sendsAnonymousStats)
    }

    func testMonoFontChoicesComeFromCoreOptions() {
        let coreOptions = monoFontOptions()

        XCTAssertEqual(AppSettings.MonoFont.allCases.map(\.rawValue), coreOptions.map(\.id))
        XCTAssertEqual(AppSettings.MonoFont.allCases.map(\.title), coreOptions.map(\.title))
    }

    func testLegacyIoskeleyNerdFontIdCanonicalizes() {
        XCTAssertEqual(
            AppSettings.MonoFont(rawValue: "ioskeleymono-nl-nerd-font")?.rawValue,
            "ioskeley-mono-nl-nerd-font"
        )
    }
}
