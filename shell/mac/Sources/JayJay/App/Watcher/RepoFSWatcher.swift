import Foundation
import JayJayCore

final class RepoFSWatcher {
    private var opSource: DispatchSourceFileSystemObject?
    private var wcSubscription: FSEventSubscription?
    private let debounceInterval: TimeInterval = 1.0
    private var lastOpFired: Date = .distantPast
    private var trailingOp: DispatchWorkItem?
    let repoPath: String

    let onOpChange: @Sendable () -> Void
    let onWorkingCopyChange: @Sendable () -> Void
    let isRelevantWorkingCopyChange: @Sendable ([String]) -> Bool

    init(
        repoPath: String,
        primaryRoot: String? = nil,
        onChange: @escaping @Sendable () -> Void,
        onWorkingCopyChange: @escaping @Sendable () -> Void = {},
        isRelevantWorkingCopyChange: @escaping @Sendable ([String]) -> Bool = { _ in true }
    ) {
        self.repoPath = repoPath
        onOpChange = onChange
        self.onWorkingCopyChange = onWorkingCopyChange
        self.isRelevantWorkingCopyChange = isRelevantWorkingCopyChange

        // Operations land in the primary repo; a secondary workspace's .jj/repo is only a pointer to it.
        let primaryRoot = primaryRoot ?? workspacePrimaryRoot(path: repoPath) ?? repoPath
        let opHeads = (primaryRoot as NSString).appendingPathComponent(".jj/repo/op_heads/heads")
        let fileDescriptor = open(opHeads, O_EVTONLY)
        if fileDescriptor >= 0 {
            let src = DispatchSource.makeFileSystemObjectSource(
                fileDescriptor: fileDescriptor,
                eventMask: [.write, .rename, .delete],
                queue: .main
            )
            src.setEventHandler { [weak self] in
                self?.fireOpChange()
            }
            src.setCancelHandler { close(fileDescriptor) }
            src.resume()
            opSource = src
        }

        wcSubscription = FSEventSubscription(path: repoPath, latency: 2.0) { [weak self] paths in
            self?.handleWorkingCopyEvents(paths)
        }
    }

    private func fireOpChange() {
        guard trailingOp == nil else { return }
        let delay = max(0, debounceInterval - Date().timeIntervalSince(lastOpFired))
        let item = DispatchWorkItem { [weak self] in
            guard let self else { return }
            trailingOp = nil
            lastOpFired = Date()
            onOpChange()
        }
        trailingOp = item
        DispatchQueue.main.asyncAfter(deadline: .now() + delay, execute: item)
    }

    func handleWorkingCopyEvents(_ paths: [String]) {
        guard isRelevantWorkingCopyChange(paths) else { return }

        // FSEvents already batches at the stream latency; another time gate can discard a delivered batch.
        DispatchQueue.main.async { [weak self] in
            self?.onWorkingCopyChange()
        }
    }

    deinit {
        opSource?.cancel()
    }
}
