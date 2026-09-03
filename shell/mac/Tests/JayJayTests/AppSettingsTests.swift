import Foundation
@testable import JayJay
import JayJayCore
import XCTest

final class AppSettingsTests: XCTestCase {
    func testAnonymousStatsDefaultOnAndPreserveExplicitOptOut() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        XCTAssertTrue(AppSettings(defaults: defaults).sendsAnonymousStats)

        defaults.set(false, forKey: AppSettings.sendsAnonymousStatsKey)
        XCTAssertFalse(AppSettings(defaults: defaults).sendsAnonymousStats)
    }

    func testSecondaryPaneWidthFallsBackToLegacyFileColumnKey() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, 260)

        defaults.set(300.0, forKey: "jayjay.fileColumnWidth")
        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, 300)

        defaults.set(340.0, forKey: "jayjay.secondaryPaneWidth")
        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, 340)
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

    func testRepositoryHistoryUsesLexicalStandardization() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        defaults.set(["/tmp/example", "/tmp/./example"], forKey: "jayjay.recentRepos")
        defaults.set("/tmp/./example", forKey: "jayjay.lastOpenedRepo")

        let settings = AppSettings(defaults: defaults)

        XCTAssertEqual(settings.recentRepos, ["/tmp/example"])
        XCTAssertEqual(settings.lastOpenedRepo, "/tmp/example")
    }
}
