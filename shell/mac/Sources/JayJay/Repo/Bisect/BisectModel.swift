import JayJayCore

struct BisectModel {
    var state: BisectState?
    var command = ""
    var activity: BisectActivity = .idle

    var isIdle: Bool {
        if case .idle = activity {
            true
        } else {
            false
        }
    }

    var commandIsRunning: Bool {
        if case .running = activity {
            true
        } else {
            false
        }
    }

    var canMark: Bool {
        isIdle(in: .evaluating) && state?.candidate != nil
    }

    var canRun: Bool {
        isIdle(in: .evaluating)
    }

    var canRestart: Bool {
        isIdle(in: .pausedRangeChanged)
    }

    var canDismiss: Bool {
        isIdle(in: .finished)
    }

    var canStop: Bool {
        guard state != nil else { return false }
        switch activity {
            case .idle, .running: return true
            case .transitioning, .stopping: return false
        }
    }

    mutating func beginTransition() -> Bool {
        guard isIdle else { return false }
        activity = .transitioning
        return true
    }

    mutating func beginCommand(_ token: JayJaySyncToken) -> Bool {
        guard isIdle else { return false }
        activity = .running(token)
        return true
    }

    mutating func beginStop() -> JayJaySyncToken? {
        switch activity {
            case .idle:
                activity = .stopping
                return nil
            case let .running(token):
                activity = .stopping
                return token
            case .transitioning, .stopping:
                return nil
        }
    }

    mutating func finishActivity() {
        activity = .idle
    }

    private func isIdle(in phase: BisectPhase) -> Bool {
        isIdle && state?.phase == phase
    }
}

enum BisectActivity {
    case idle
    case transitioning
    case running(JayJaySyncToken)
    case stopping
}
