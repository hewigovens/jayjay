import JayJayCore
import JayJayDiffUI

struct DiffContextExpansionIdentity: Equatable, Sendable {
    let compareFromRev: String?
    let commitId: String?
    let rev: String?
    let path: String
    let ignoreWhitespace: Bool
    let projectionMode: String
}

extension DiffSection {
    /// The displayed diff's basis, not the live controls: during a reload the old render stays visible, and expanding it must supersede when the replacement installs.
    var contextExpansionIdentity: DiffContextExpansionIdentity? {
        loadedDiff?.identity
    }

    func expandAllContext() {
        guard let region = loadedDiff?.fileDiff?.lines.compactMap(\.contextRegion).first
        else { return }
        expandContext(DiffContextExpansionRequest(regionId: region.id, action: .showAllRegions))
    }

    func expandContext(_ request: DiffContextExpansionRequest) {
        guard let current = loadedDiff,
              let diff = current.fileDiff,
              Self.requestTargetsAvailableRegion(request, in: diff)
        else { return }
        guard let identity = contextExpansionIdentity else { return }
        guard let attempt = contextExpansion.start(request) else { return }
        let oldContent = current.content.oldText
        let newContent = current.content.newText

        Task {
            let prepared = await Task.detached(priority: .userInitiated) {
                prepareContextExpansion(
                    request: request,
                    attempt: attempt,
                    diff: diff,
                    oldContent: oldContent,
                    newContent: newContent
                )
            }.value

            guard Self.shouldAcceptContextExpansion(
                requestIdentity: identity,
                currentIdentity: contextExpansionIdentity,
                requestGeneration: attempt.generation,
                currentGeneration: contextExpansion.generation
            ) else {
                if contextExpansion.generation == attempt.generation {
                    resetContextExpansion()
                }
                return
            }

            switch prepared {
                case let .success(prepared):
                    guard var current = loadedDiff else {
                        resetContextExpansion()
                        return
                    }

                    current.fileDiff = prepared.diff
                    current.displayLines = prepared.displayLines
                    current.displayGroups = prepared.displayGroups
                    selectedLineRange = nil
                    let pending = contextExpansion.complete(
                        session: prepared.session,
                        revealFeedback: prepared.feedback
                    )
                    loadedDiff = current
                    refreshActiveNotes()
                    Task {
                        try? await Task.sleep(for: .milliseconds(300))
                        contextExpansion.clearRevealFeedback(generation: attempt.generation)
                    }

                    if let pending {
                        expandContext(pending)
                    }
                case let .failure(error):
                    contextExpansion.fail(message: contextExpansionErrorMessage(error))
            }
        }
    }

    /// A queued expand-all can outlive the region id it was created with; it stays valid while any region remains.
    nonisolated static func requestTargetsAvailableRegion(
        _ request: DiffContextExpansionRequest,
        in diff: FileDiff
    ) -> Bool {
        if request.action == .showAllRegions {
            return diff.lines.contains(where: { $0.contextRegion != nil })
        }
        return diff.lines.contains(where: { $0.contextRegion?.id == request.regionId })
    }

    nonisolated static func shouldAcceptContextExpansion(
        requestIdentity: DiffContextExpansionIdentity,
        currentIdentity: DiffContextExpansionIdentity?,
        requestGeneration: UInt64,
        currentGeneration: UInt64
    ) -> Bool {
        requestIdentity == currentIdentity && requestGeneration == currentGeneration
    }
}

private func contextExpansionErrorMessage(_ error: ContextExpansionError) -> String {
    switch error {
        case .UnknownRegion:
            "The diff changed before its context could be expanded. Refresh and try again."
        case .InvalidLineCount, .InvalidRegion, .MissingSourceLine, .SessionUnavailable:
            "This context could not be expanded. Refresh the diff and try again."
    }
}

private struct PreparedContextExpansion: Sendable {
    let session: ExpandableDiff
    let diff: FileDiff
    let displayLines: [DiffLine]
    let displayGroups: [ChangeGroup]
    let feedback: DiffContextRevealFeedback?
}

private func prepareContextExpansion(
    request: DiffContextExpansionRequest,
    attempt: (generation: UInt64, session: ExpandableDiff?),
    diff: FileDiff,
    oldContent: String,
    newContent: String
) -> Result<PreparedContextExpansion, ContextExpansionError> {
    let session = attempt.session
        ?? makeExpandableDiff(diff: diff, oldContent: oldContent, newContent: newContent)
    let result: ContextExpansionResult
    do {
        result = switch request.action {
            case let .showMore(lineCount):
                try session.expand(regionId: request.regionId, expansion: .showMore(lineCount: lineCount))
            case .showAll:
                try session.expand(regionId: request.regionId, expansion: .showAll)
            case .showAllRegions:
                try session.expandAll()
        }
    } catch let error as ContextExpansionError {
        return .failure(error)
    } catch {
        return .failure(.SessionUnavailable)
    }

    let displayLines = diffDisplayLines(lines: result.diff.lines)
    let insertedStart = Int(result.inserted.start)
    let insertedEnd = insertedStart + Int(result.inserted.count)
    let newLineStart: UInt32? = if insertedStart >= 0, insertedEnd <= result.diff.lines.count {
        result.diff.lines[insertedStart ..< insertedEnd].compactMap(\.newLineNo).min()
    } else {
        nil
    }
    let feedback = newLineStart.map {
        DiffContextRevealFeedback(
            generation: attempt.generation,
            newLines: LineSpan(start: $0, count: result.inserted.count)
        )
    }
    return .success(PreparedContextExpansion(
        session: session,
        diff: result.diff,
        displayLines: displayLines,
        displayGroups: changeGroups(lines: displayLines),
        feedback: feedback
    ))
}
