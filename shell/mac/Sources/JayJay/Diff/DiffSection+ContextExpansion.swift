import JayJayCore

extension DiffSection {
    /// The displayed diff's basis, not the live controls: during a reload the old render stays visible, and expanding it must supersede when the replacement installs.
    var contextExpansionBasis: String? {
        loadedDiff?.basis
    }

    func expandAllContext() {
        guard loadedDiff?.fileDiff?.lines.contains(where: { $0.contextRegion != nil }) == true else { return }
        expandContext(.allRegions)
    }

    func expandContext(_ request: ContextExpansionRequest) {
        guard let current = loadedDiff,
              let diff = current.fileDiff,
              let basis = current.basis,
              let attempt = contextExpansion.begin(basis: basis, request: request)
        else { return }
        let source = attempt.needsSource()
            ? ContextExpansionSource(
                diff: diff,
                oldContent: current.content.oldText,
                newContent: current.content.newText
            )
            : nil
        let session = contextExpansion
        let generation = attempt.generation()

        Task {
            let expanded = await Task.detached(priority: .userInitiated) {
                attempt.run(source: source)
            }.value
            guard let outcome = expanded else { return }

            let currentBasis = contextExpansionBasis
            let prepared = await Task.detached(priority: .userInitiated) {
                PreparedContextExpansion(session.finish(basis: currentBasis ?? "", outcome: outcome))
            }.value
            guard session.isCurrent(basis: contextExpansionBasis ?? "", generation: generation) else { return }
            install(prepared)
        }
    }

    private func install(_ prepared: PreparedContextExpansion) {
        switch prepared.finish {
            case .discarded:
                break
            case let .applied(diff, reveal, selectionGeneration, next):
                guard var current = loadedDiff else { return }
                current.fileDiff = diff
                current.displayLines = prepared.displayLines
                current.displayGroups = prepared.displayGroups
                selectedLineRange = nil
                loadedDiff = current
                contextExpansionDisplay.errorMessage = nil
                contextExpansionDisplay.selectionGeneration = selectionGeneration
                contextExpansionDisplay.revealFeedback = reveal
                refreshActiveNotes()
                if let reveal {
                    Task {
                        try? await Task.sleep(for: .milliseconds(300))
                        contextExpansionDisplay.clearRevealFeedback(generation: reveal.generation)
                    }
                }
                if let next {
                    expandContext(next)
                }
            case let .failed(message):
                contextExpansionDisplay.errorMessage = message
        }
    }
}

/// The core outcome with its display projection, both built off the main thread because each is O(diff bytes).
private struct PreparedContextExpansion: Sendable {
    let finish: ContextExpansionFinish
    let displayLines: [DiffLine]
    let displayGroups: [ChangeGroup]

    init(_ finish: ContextExpansionFinish) {
        self.finish = finish
        guard case let .applied(diff, _, _, _) = finish else {
            displayLines = []
            displayGroups = []
            return
        }
        let lines = diffDisplayLines(lines: diff.lines)
        displayLines = lines
        displayGroups = changeGroups(lines: lines)
    }
}
