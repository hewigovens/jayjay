import AppKit
import Foundation

struct ScenePlayer {
    let session: AppSession
    let rows: RowTable?
    let cues: CueLog

    func play(_ steps: [Step]) async throws {
        for (index, step) in steps.enumerated() {
            do {
                try SafetyStop.checkRequested()
                try await perform(step)
                try requireFocus()
            } catch let stop as SafetyStop {
                throw SafetyStop("step \(index + 1) (\(step.summary)): \(stop)")
            } catch {
                throw Failure("step \(index + 1) (\(step.summary)): \(error)")
            }
        }
    }

    private func perform(_ step: Step) async throws {
        switch step.action {
            case let .click(target):
                try await click(target, step: step, button: .left, count: 1)
            case let .doubleClick(target):
                try await click(target, step: step, button: .left, count: 2)
            case let .rightClick(target):
                try await click(target, step: step, button: .right, count: 1)
            case let .key(combo):
                try requireFocus()
                TakeCleanup.shared.willPostKeys()
                try combo.post(check: requireFocus)
                cues.add(step.label ?? combo.text, at: nil)
            case let .paste(text):
                try requireFocus()
                TakeCleanup.shared.willPostKeys()
                TakeCleanup.shared.putOnPasteboard(text)
                try KeyCombo.paste.post(check: requireFocus)
                cues.add(step.label ?? "paste", at: nil)
            case let .wait(target):
                _ = try await find(target, within: step.timeout)
            case let .waitGone(target):
                let deadline = Date().addingTimeInterval(step.timeout)
                // A busy app answers nothing, which would read as gone.
                while try session.app.summary().role == nil || target.resolve(in: session.app, rows: rows) != nil {
                    guard Date() < deadline else { throw Failure("still on screen after \(step.timeout) s") }
                    try await hold(for: .milliseconds(150))
                }
            case let .beat(seconds):
                try await hold(for: .seconds(seconds))
        }
    }

    private func click(_ target: Target, step: Step, button: CGMouseButton, count: Int) async throws {
        let hit = try await find(target, within: step.timeout)
        var cued = false
        try await Pointer.click(at: hit.point, button: button, count: count) {
            try requireFocus()
            if let covering = session.coveringApp(at: hit.point, over: hit.element) {
                throw SafetyStop("\(covering) covers it; stopped without clicking")
            }
            if !cued {
                cues.add(step.label ?? target.name, at: hit.point)
                cued = true
            }
        }
    }

    private func requireFocus() throws {
        guard session.hasFocus else { throw SafetyStop("the app lost focus; stopped without clicking or typing") }
    }

    private func hold(for duration: Duration) async throws {
        let end = ContinuousClock.now + duration
        while true {
            try SafetyStop.checkRequested()
            try requireFocus()
            let left = end - .now
            guard left > .zero else { return }
            try await Task.sleep(for: min(left, .milliseconds(100)))
        }
    }

    private func find(_ target: Target, within timeout: TimeInterval) async throws -> (element: AXElement, point: CGPoint) {
        let deadline = Date().addingTimeInterval(timeout)
        while true {
            if let hit = try target.resolve(in: session.app, rows: rows) {
                return hit
            }
            guard Date() < deadline else { throw Failure("not on screen after \(timeout) s") }
            try await hold(for: .milliseconds(150))
        }
    }
}
