import Foundation
@testable import JayJay
import JayJayCore
import JayJayDiffUI
import SwiftUI
import XCTest

final class AppSettingsTests: XCTestCase {
    func testFontSettingsSurviveReload() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)
        settings.interfaceFontFamily = try XCTUnwrap(AppSettings.FontFamily(rawValue: "Courier New"))
        settings.interfaceFontSize = 15
        settings.codeFontFamily = try XCTUnwrap(AppSettings.FontFamily(rawValue: "menlo"))
        settings.codeFontSize = 20
        settings.codeLineSpacing = 1.4
        settings.codeLigatures = false
        XCTAssertEqual(AppSettings(defaults: defaults).typography, settings.typography)
    }

    func testPreSplitSettingsKeepCodeFontAndResetInterfaceFamily() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }
        defaults.set("menlo", forKey: "jayjay.fontFamily")
        defaults.set("15", forKey: "jayjay.fontSize")

        let migrated = AppSettings(defaults: defaults)
        XCTAssertEqual(migrated.typography, try Typography(
            interfaceFamily: .system,
            interfaceSize: 15,
            codeFamily: XCTUnwrap(AppSettings.FontFamily(rawValue: "menlo")),
            codeSize: 15
        ))
        migrated.codeFontSize = 18
        XCTAssertEqual(AppSettings(defaults: defaults).interfaceFontSize, 15, "the seeded interface size must not follow later code-size changes")
    }

    func testZoomMovesBothSizesWithinBoundsAndResetRestoresDefaults() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)
        settings.interfaceFontSize = AppSettings.fontSizeRange.upperBound - 1
        settings.codeFontSize = 12
        settings.zoomIn()
        settings.zoomIn()
        XCTAssertEqual([settings.interfaceFontSize, settings.codeFontSize], [AppSettings.fontSizeRange.upperBound, 14])
        settings.resetZoom()
        XCTAssertEqual([settings.interfaceFontSize, settings.codeFontSize], [12, 12])
        settings.codeFontSize = AppSettings.fontSizeRange.lowerBound
        settings.zoomOut()
        XCTAssertEqual([settings.interfaceFontSize, settings.codeFontSize], [11, AppSettings.fontSizeRange.lowerBound])

        settings.interfaceFontFamily = try XCTUnwrap(AppSettings.FontFamily(rawValue: "Courier New"))
        settings.codeLigatures = false
        settings.resetFonts()
        XCTAssertEqual([settings.interfaceFontSize, settings.codeFontSize], [12, 12])
        XCTAssertEqual([settings.interfaceFontFamily, settings.codeFontFamily], [.system, .system])
        XCTAssertTrue(settings.codeLigatures)
    }

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

        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, Double(PaneLayout.secondaryPaneDefault))

        defaults.set(280.0, forKey: "jayjay.fileColumnWidth")
        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, 280)

        defaults.set(340.0, forKey: "jayjay.secondaryPaneWidth")
        XCTAssertEqual(AppSettings(defaults: defaults).secondaryPaneWidth, 340)
    }

    func testRecentRepositoriesPanelFollowsTheListUntilToggled() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        XCTAssertFalse(AppSettings(defaults: defaults).showsRecentRepositoriesPanel)

        let settings = AppSettings(defaults: defaults)
        settings.addRecentRepo("/tmp/one")
        XCTAssertTrue(settings.showsRecentRepositoriesPanel)
        settings.showsRecentRepositoriesPanel = false
        XCTAssertFalse(AppSettings(defaults: defaults).showsRecentRepositoriesPanel)

        settings.removeRecentRepo("/tmp/one")
        XCTAssertFalse(settings.showsRecentRepositoriesPanel)
        settings.addRecentRepo("/tmp/two")
        XCTAssertTrue(AppSettings(defaults: defaults).showsRecentRepositoriesPanel)
    }

    func testPreferencesBecomeFamilyNamesThatResolveToInstalledFonts() {
        XCTAssertEqual(AppSettings.FontFamily(rawValue: "ioskeleymono-nl-nerd-font")?.rawValue, "IoskeleyMonoNL Nerd Font")
        XCTAssertEqual(AppSettings.FontFamily(rawValue: "jetbrains-mono")?.rawValue, "JetBrains Mono")
        XCTAssertEqual(AppSettings.FontFamily(rawValue: "system"), .system)
        XCTAssertNil(AppSettings.FontFamily(rawValue: ""))
        XCTAssertEqual(AppSettings.FontFamily(rawValue: "Courier New")?.nsFont(size: 12).familyName, "Courier New")
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

    func testDiffPaletteResolutionAndWallpaperTintingRule() throws {
        let suite = "AppSettingsTests.\(UUID().uuidString)"
        let defaults = try XCTUnwrap(UserDefaults(suiteName: suite))
        defer { defaults.removePersistentDomain(forName: suite) }

        let settings = AppSettings(defaults: defaults)
        XCTAssertEqual(settings.diffPalettes, .standard)
        XCTAssertTrue(settings.usesDefaultDarkDiffPalette)
        XCTAssertFalse(settings.effectiveTintWindowWithWallpaper)

        settings.tintWindowWithWallpaper = true
        XCTAssertTrue(settings.effectiveTintWindowWithWallpaper)
        XCTAssertEqual(settings.readingBackground(.dark), .clear)

        settings.diffPaletteDark = "nord"
        XCTAssertNotEqual(settings.diffPalettes.dark, DiffPalettes.standard.dark)
        XCTAssertFalse(settings.usesDefaultDarkDiffPalette)
        XCTAssertFalse(settings.effectiveTintWindowWithWallpaper)
        XCTAssertNotEqual(settings.readingBackground(.dark), .clear)
        XCTAssertTrue(settings.tintWindowWithWallpaper)
        XCTAssertEqual(defaults.object(forKey: "jayjay.tintWindowWithWallpaper") as? Bool, true)

        settings.diffPaletteDark = AppSettings.defaultDarkDiffPaletteId
        XCTAssertTrue(settings.usesDefaultDarkDiffPalette)
        XCTAssertTrue(settings.effectiveTintWindowWithWallpaper)
        XCTAssertEqual(settings.readingBackground(.dark), .clear)

        XCTAssertNotEqual(settings.readingBackground(.light), .clear, "tinting is dark-mode only")

        defaults.set("future-dark-palette", forKey: "jayjay.diffPaletteDark")
        let downgraded = AppSettings(defaults: defaults)
        XCTAssertEqual(downgraded.diffPaletteDark, "future-dark-palette")
        XCTAssertEqual(defaults.string(forKey: "jayjay.diffPaletteDark"), "future-dark-palette")
        XCTAssertEqual(downgraded.diffPalettes.dark, DiffPalettes.standard.dark)
        XCTAssertTrue(downgraded.usesDefaultDarkDiffPalette)
        XCTAssertTrue(downgraded.effectiveTintWindowWithWallpaper)
    }
}
