import JayJayCore
import Observation

@MainActor
@Observable
final class MergeScrollCoordinator {
    var isRaw = false {
        didSet {
            guard isRaw != oldValue else { return }
            for pane in [MergePane.left, .base, .right] {
                if let anchor = anchors[pane] {
                    lastScroll = .text(pane, anchor.centerLine)
                    break
                }
            }
        }
    }

    private(set) var visibleHunk: UInt32?

    private enum Position {
        case text(MergePane, Double)
        case hunk(UInt32)
    }

    @ObservationIgnored private var hunks: [UInt32] = []
    @ObservationIgnored private var map: MergeScrollMap?
    @ObservationIgnored private var anchors: [MergePane: MergeTextScrollAnchor] = [:]
    @ObservationIgnored private var lastScroll: Position?
    @ObservationIgnored private var isApplying = false
    @ObservationIgnored private var resultIsCurrent = false
    @ObservationIgnored private var resultNeedsRestore = false

    func register(_ pane: MergePane, anchor: MergeTextScrollAnchor) {
        anchors[pane] = anchor
        if pane == .result {
            resultNeedsRestore = true
        }
        Task { @MainActor [weak self, weak anchor] in
            await Task.yield()
            guard let self, let anchor, anchors[pane] === anchor else { return }
            if let lastScroll {
                apply(lastScroll, only: pane)
            }
        }
    }

    func unregister(_ pane: MergePane, anchor: MergeTextScrollAnchor) {
        if anchors[pane] === anchor {
            anchors[pane] = nil
            if pane == .result {
                resultNeedsRestore = false
            }
        }
    }

    func update(map: MergeScrollMap, hunks: [UInt32] = []) {
        let reconcileResult = self.map != nil && !resultIsCurrent && isRaw
        self.map = map
        self.hunks = hunks
        resultIsCurrent = true
        if isRaw, resultNeedsRestore {
            synchronize()
        } else if reconcileResult {
            didScroll(.result)
        }
    }

    func reveal(hunk: UInt32?) {
        guard !isRaw, let hunk else { return }
        lastScroll = .hunk(hunk)
        apply(.hunk(hunk))
    }

    func invalidateResult() {
        resultIsCurrent = false
    }

    func didScroll(_ pane: MergePane) {
        guard !isApplying, map != nil, let anchor = anchors[pane] else { return }
        if pane == .result {
            guard isRaw else { return }
            resultNeedsRestore = false
            guard resultIsCurrent else { return }
        } else if !resultIsCurrent {
            resultNeedsRestore = true
        }
        lastScroll = .text(pane, anchor.centerLine)
        synchronize()
    }

    func didScroll(hunk: UInt32) {
        guard !isRaw, !isApplying else { return }
        visibleHunk = hunk
        lastScroll = .hunk(hunk)
        synchronize()
    }

    private func synchronize() {
        guard let lastScroll else { return }
        apply(lastScroll)
    }

    private func apply(_ position: Position, only restoredPane: MergePane? = nil) {
        guard !isApplying, let map else { return }
        isApplying = true
        defer { isApplying = false }
        switch position {
            case let .text(pane, line):
                guard pane != .result || (isRaw && resultIsCurrent) else { return }
                for (follower, target) in anchors where (restoredPane == nil || follower == restoredPane) && (follower != pane || follower == restoredPane) && (follower != .result || (isRaw && resultIsCurrent)) {
                    target.scroll(to: map.mapLine(from: pane, to: follower, line: line))
                    if follower == .result {
                        resultNeedsRestore = false
                    }
                }
                if !isRaw, restoredPane == nil {
                    visibleHunk = hunks.min {
                        abs((map.hunkLine(pane: pane, index: $0) ?? .infinity) - line)
                            < abs((map.hunkLine(pane: pane, index: $1) ?? .infinity) - line)
                    }
                }
            case let .hunk(hunk):
                if restoredPane == nil {
                    visibleHunk = hunk
                }
                for (pane, anchor) in anchors where pane != .result && (restoredPane == nil || pane == restoredPane) {
                    if let line = map.hunkLine(pane: pane, index: hunk) {
                        anchor.scroll(to: line)
                    }
                }
        }
    }
}
