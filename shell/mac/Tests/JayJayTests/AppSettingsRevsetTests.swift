@testable import JayJay
import XCTest

final class AppSettingsRevsetTests: XCTestCase {
    func testSavesAndReloadsRevsets() {
        let defaults = makeDefaults()
        let settings = AppSettings(defaults: defaults)

        settings.saveRevset(name: "Mine touching docs", expression: "mine() & files('docs/**')")

        let reloaded = AppSettings(defaults: defaults)
        XCTAssertEqual(reloaded.savedRevsets.count, 1)
        XCTAssertEqual(reloaded.savedRevsets.first?.name, "Mine touching docs")
        XCTAssertEqual(reloaded.savedRevsets.first?.expression, "mine() & files('docs/**')")
    }

    func testSavingSameExpressionReplacesExistingRevset() {
        let settings = AppSettings(defaults: makeDefaults())

        settings.saveRevset(name: "Old", expression: "heads(all())")
        settings.saveRevset(name: "New", expression: "heads(all())")

        XCTAssertEqual(settings.savedRevsets.count, 1)
        XCTAssertEqual(settings.savedRevsets.first?.name, "New")
    }

    func testRemoveSavedRevset() throws {
        let settings = AppSettings(defaults: makeDefaults())
        settings.saveRevset(name: "Scratch", expression: "@")
        let id = try XCTUnwrap(settings.savedRevsets.first?.id)

        settings.removeSavedRevset(id: id)

        XCTAssertTrue(settings.savedRevsets.isEmpty)
    }

    func testSavesAndReloadsEvologDisplayPreferences() {
        let defaults = makeDefaults()
        let settings = AppSettings(defaults: defaults)

        settings.evologHideSnapshots = true
        settings.evologCollapseSnapshotRuns = false

        let reloaded = AppSettings(defaults: defaults)
        XCTAssertTrue(reloaded.evologHideSnapshots)
        XCTAssertFalse(reloaded.evologCollapseSnapshotRuns)
    }

    private func makeDefaults() -> UserDefaults {
        let suiteName = "dev.hewig.jayjay.tests.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        defaults.removePersistentDomain(forName: suiteName)
        return defaults
    }
}
