import Foundation
import JayJayCore

extension RepoViewModel {
    @discardableResult
    func startBisect(good: String, bad: String, range: String) -> Bool {
        guard bisect.isIdle else { return false }
        do {
            let composed = try composeBisectRange(good: good, bad: bad, range: range)
            runBisect { try $0.startBisect(range: composed) }
            return true
        } catch {
            self.error = error.friendlyDescription
            return false
        }
    }

    func startBisectBetween(first: String, second: String) {
        runBisect { try $0.startBisectBetween(first: first, second: second) }
    }

    func markBisect(_ verdict: BisectVerdict) {
        guard bisect.canMark else { return }
        guard let commitId = bisect.state?.candidate?.commitId else { return }
        runBisect { try $0.bisectMark(commitId: commitId, verdict: verdict) }
    }

    func restartBisect() {
        guard bisect.canRestart else { return }
        runBisect { try $0.bisectRestart() }
    }

    func dismissBisect() {
        guard bisect.canDismiss else { return }
        guard bisect.beginTransition() else { return }
        performBisect(selecting: { _ in "@" }, { try $0.bisectDismiss() })
    }

    func stopBisect() {
        guard bisect.canStop else { return }
        let token = bisect.beginStop()
        token?.cancel()
        performBisect(selecting: { _ in "@" }, { try $0.bisectStop() })
    }

    func runBisectCommand(toEnd: Bool) {
        guard bisect.canRun else { return }
        let command = bisect.command.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !command.isEmpty else {
            error = "Enter a command to run."
            return
        }
        let sync = repo.syncToken()
        guard bisect.beginCommand(sync) else { return }
        performBisect(selecting: { $0.map(Self.bisectSelection) ?? "@" }, { repo in
            if toEnd {
                return try repo.bisectRunToEnd(command: command, sync: sync)
            }
            return try repo.bisectRunStep(command: command, sync: sync)
        })
    }

    private func runBisect(_ action: @escaping RepoOperation<BisectState>) {
        guard bisect.beginTransition() else { return }
        performBisect(selecting: Self.bisectSelection, action)
    }

    private func performBisect<Result>(
        selecting: @escaping (Result) -> String,
        _ action: @escaping RepoOperation<Result>
    ) {
        performResult(
            selectingResult: selecting,
            afterRefresh: { viewModel, _ in viewModel.bisect.finishActivity() },
            onFailure: { viewModel, error in
                viewModel.bisect.finishActivity()
                viewModel.present(error: error)
            },
            action
        )
    }

    private static func bisectSelection(_ state: BisectState) -> String {
        if state.phase == .finished {
            return state.result?.bad.first?.changeId ?? "@"
        }
        return state.candidate?.changeId ?? "@"
    }
}
