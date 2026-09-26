import CoreGraphics
import Foundation

struct Take {
    let scene: Scene
    let options: RunOptions

    /// Writes under a partial name first, so a failed retake never replaces a good clip.
    func record() async throws {
        let out = URL(fileURLWithPath: options.out, isDirectory: true)
        let bundle = URL(fileURLWithPath: options.app, isDirectory: true).standardizedFileURL
        if let setup = scene.setup {
            try Shell.run("/usr/bin/env", setup.map { $0.replacingOccurrences(of: "${app}", with: bundle.path) })
        }
        let rows = try scene.rows.map { try RowTable(contentsOf: URL(fileURLWithPath: $0)) }
        try await waitForIdleInput()
        let session = try await AppSession.launch(bundle: bundle, scene: scene, log: out.appendingPathComponent("\(scene.name).log"))
        TakeCleanup.shared.launched(session.pid)
        let partial = options.capture ? Outputs(out: out, name: "\(scene.name).partial") : nil
        partial?.remove()
        var played: Result<Void, Error>
        do {
            played = try await .success(play(in: session, rows: rows, writingTo: partial))
        } catch {
            played = .failure(error)
        }
        TakeCleanup.shared.run(quittingApp: false)
        if !options.keepOpen {
            await AppSession.quit(session.pid)
        }
        if case .success = played, SafetyStop.isRequested {
            played = .failure(SafetyStop("interrupted while finishing the take"))
        }
        partial?.move(to: Outputs(out: out, name: (try? played.get()) == nil ? "\(scene.name).failed" : scene.name))
        try played.get()
    }

    private func play(in session: AppSession, rows: RowTable?, writingTo outputs: Outputs?) async throws {
        var clip = try await session.waitForWindow()
        if let frame = scene.window, frame != clip {
            try session.place(frame)
            clip = try await session.waitForWindow()
            if frame != clip {
                print("\(scene.name): the window settled at \(clip) instead of \(frame)")
            }
        }
        try await waitForIdleInput()
        try await session.activate()
        let inset = clip.insetBy(dx: 24, dy: 24)
        guard let window = session.mainWindow else { throw Failure("the app has no window") }
        for point in [CGPoint(x: clip.midX, y: clip.midY), inset.origin, CGPoint(x: inset.maxX, y: inset.minY), CGPoint(x: inset.minX, y: inset.maxY), CGPoint(x: inset.maxX, y: inset.maxY)] {
            if let covering = session.coveringApp(at: point, over: window) {
                throw SafetyStop("\(covering) covers the app's window; nothing may sit in front of it while recording")
            }
        }
        var recorder: ScreenRecorder?
        if let outputs {
            recorder = try await ScreenRecorder.start(rect: clip, output: outputs.clip)
        }
        let cues = CueLog(clip: clip, start: recorder?.startedAt ?? Date())
        var failure: Error?
        do {
            try await ScenePlayer(session: session, rows: rows, cues: cues).play(scene.prelude + scene.steps)
        } catch {
            failure = error
        }
        do {
            try await recorder?.stop()
            try outputs.map { try cues.write(to: $0.cues) }
        } catch {
            guard let failure else { throw error }
            print("\(scene.name): could not finish the clip: \(error)")
            throw failure
        }
        if let failure {
            throw failure
        }
    }

    private func waitForIdleInput() async throws {
        let anyInput = CGEventType(rawValue: ~0)!
        let deadline = Date().addingTimeInterval(60)
        var announced = false
        while CGEventSource.secondsSinceLastEventType(.hidSystemState, eventType: anyInput) < 2 {
            try SafetyStop.checkRequested()
            guard Date() < deadline else { throw SafetyStop("the keyboard and mouse stayed busy for a minute; record when the Mac is free") }
            if !announced {
                print("\(scene.name): waiting for the keyboard and mouse to be idle")
                announced = true
            }
            try await Task.sleep(for: .milliseconds(250))
        }
    }

    private struct Outputs {
        let clip: URL
        let cues: URL

        init(out: URL, name: String) {
            clip = out.appendingPathComponent("\(name).mov")
            cues = out.appendingPathComponent("\(name).cues.jsonl")
        }

        func remove() {
            for url in [clip, cues] {
                try? FileManager.default.removeItem(at: url)
            }
        }

        func move(to destination: Outputs) {
            for (from, to) in [(clip, destination.clip), (cues, destination.cues)] where FileManager.default.fileExists(atPath: from.path) {
                try? FileManager.default.removeItem(at: to)
                try? FileManager.default.moveItem(at: from, to: to)
            }
        }
    }
}
