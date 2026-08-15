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

    func testWorkspaceSidebarDefaultsAndPersistence() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)
        XCTAssertTrue(settings.workspaceSidebarVisible)
        XCTAssertEqual(settings.workspaceSidebarWidth, 260)

        settings.workspaceSidebarVisible = false
        settings.workspaceSidebarWidth = 240
        let reloaded = AppSettings(defaults: defaults)
        XCTAssertFalse(reloaded.workspaceSidebarVisible)
        XCTAssertEqual(reloaded.workspaceSidebarWidth, 240)
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
