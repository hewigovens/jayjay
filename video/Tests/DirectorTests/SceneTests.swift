import CoreGraphics
@testable import Director
import XCTest

final class SceneTests: XCTestCase {
    private var folder: URL!

    override func setUpWithError() throws {
        folder = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
        try FileManager.default.createDirectory(at: folder, withIntermediateDirectories: true)
    }

    override func tearDownWithError() throws {
        try FileManager.default.removeItem(at: folder)
    }

    func testSceneOverridesItsBaseAndMergesDefaults() throws {
        try write("_base.json", #"{"repo": "/tmp/base", "window": [80, 80, 1600, 900], "defaults": {"a": true, "b": 12}, "prelude": [{"beat": 1}]}"#)
        try write("split.json", #"{"extends": "_base.json", "defaults": {"b": "{{0, 0}, {10, 10}}"}, "steps": [{"menu": "Show in Graph"}, {"key": "cmd+shift+o"}]}"#)

        let scene = try Scene.load(folder.appendingPathComponent("split.json"))

        XCTAssertEqual(scene.name, "split")
        XCTAssertEqual(scene.repo, "/tmp/base")
        XCTAssertEqual(scene.window, CGRect(x: 80, y: 80, width: 1600, height: 900))
        XCTAssertEqual(scene.defaults["a"]?.argument, "YES")
        XCTAssertEqual(scene.defaults["b"]?.argument, #""{{0, 0}, {10, 10}}""#)
        XCTAssertEqual(scene.prelude.count, 1)
        XCTAssertEqual(scene.steps.map(\.summary), [#"click menu item "Show in Graph""#, "key cmd+shift+o"])
    }

    func testCheckedInScenesLoad() throws {
        let video = URL(fileURLWithPath: #filePath).deletingLastPathComponent().appendingPathComponent("../..").standardizedFileURL
        let files = ["examples", "scenes"].flatMap { folder in
            (try? FileManager.default.contentsOfDirectory(at: video.appendingPathComponent(folder), includingPropertiesForKeys: nil)) ?? []
        }.filter { $0.pathExtension == "json" }
        XCTAssertFalse(files.isEmpty)
        for file in files {
            XCTAssertNoThrow(try Scene.load(file), file.lastPathComponent)
        }
    }

    func testMisspelledKeyFailsWithItsPath() throws {
        try write("typo.json", #"{"steps": [{"click": {"lable": "Cancel"}}]}"#)

        XCTAssertThrowsError(try Scene.load(folder.appendingPathComponent("typo.json"))) { error in
            XCTAssertTrue("\(error)".hasPrefix("typo.json: steps[0].click: unknown key lable"), "\(error)")
        }
    }

    func testExtendsCycleFails() throws {
        try write("a.json", #"{"extends": "b.json"}"#)
        try write("b.json", #"{"extends": "a.json"}"#)

        XCTAssertThrowsError(try Scene.load(folder.appendingPathComponent("a.json"))) { error in
            XCTAssertTrue("\(error)".contains("extends itself"), "\(error)")
        }
    }

    func testScenesSharingAnOutputNameFail() throws {
        for (appearance, name) in [("light", "Demo"), ("dark", "demo")] {
            try FileManager.default.createDirectory(at: folder.appendingPathComponent(appearance), withIntermediateDirectories: true)
            try write("\(appearance)/\(name).json", "{}")
        }

        XCTAssertThrowsError(try Scene.load([folder.appendingPathComponent("light/Demo.json"), folder.appendingPathComponent("dark/demo.json")])) { error in
            XCTAssertTrue("\(error)".contains("named demo"), "\(error)")
        }
    }

    func testSceneNamedLikeATakeOutputFails() throws {
        try write("demo.partial.json", "{}")

        XCTAssertThrowsError(try Scene.load([folder.appendingPathComponent("demo.partial.json")])) { error in
            XCTAssertTrue("\(error)".contains("partial or failed output"), "\(error)")
        }
    }

    func testExecutablePathUnderTmpMatchesTheKernelsPrivatePath() throws {
        let file = URL(fileURLWithPath: "/tmp/director-\(UUID().uuidString)")
        try Data().write(to: file)
        defer { try? FileManager.default.removeItem(at: file) }

        XCTAssertEqual(AppSession.realPath(file.path), "/private" + file.path)
    }

    func testLaneOfADivergentChangeFailsInsteadOfTimingOut() throws {
        try write("rows.tsv", "297ae59ccaba\texperiment: score routes by comfort\n")
        let target = try JSONDecoder().decode(Target.self, from: Data(#"{"lane": "comfort"}"#.utf8))

        XCTAssertThrowsError(try target.resolve(in: AXElement.application(getpid()), rows: RowTable(contentsOf: folder.appendingPathComponent("rows.tsv")))) { error in
            XCTAssertTrue("\(error)".contains("divergent"), "\(error)")
        }
    }

    func testStepOptionThatDoesNothingFails() throws {
        try write("beat.json", #"{"steps": [{"beat": 1, "label": "pause"}]}"#)

        XCTAssertThrowsError(try Scene.load(folder.appendingPathComponent("beat.json"))) { error in
            XCTAssertTrue("\(error)".contains("beat takes no label"), "\(error)")
        }
    }

    func testKeyCombosCarryTheFlagsAHardwareKeyboardSends() throws {
        XCTAssertEqual(try KeyCombo("cmd+shift+o").flags, [.maskCommand, .maskShift])
        XCTAssertEqual(try KeyCombo("cmd+down").flags, [.maskCommand, .maskNumericPad, .maskSecondaryFn])

        XCTAssertThrowsError(try KeyCombo("hyper+o"))
        XCTAssertThrowsError(try KeyCombo("cmd+"))
    }

    func testLaunchValuesQuoteEverythingButPlainTokens() {
        XCTAssertEqual(LaunchValue.string("dark").argument, "dark")
        XCTAssertEqual(LaunchValue.string("").argument, #""""#)
        XCTAssertEqual(LaunchValue.string(#"say "hi""#).argument, #""say \"hi\"""#)
        XCTAssertEqual(LaunchValue.number(14).argument, "14")
        XCTAssertEqual(LaunchValue.bool(false).argument, "NO")
    }

    private func write(_ name: String, _ json: String) throws {
        try json.write(to: folder.appendingPathComponent(name), atomically: true, encoding: .utf8)
    }
}
