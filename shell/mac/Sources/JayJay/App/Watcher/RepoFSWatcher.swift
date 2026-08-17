import Foundation
import JayJayCore

/// Watches jj operation heads and working copy for changes.
final class RepoFSWatcher {
    private var opSource: DispatchSourceFileSystemObject?
    private var wcStream: FSEventStreamRef?
    private let debounceInterval: TimeInterval = 1.0
    private var lastOpFired: Date = .distantPast
    private var lastWCFired: Date = .distantPast
    let repoPath: String

    let onOpChange: @Sendable () -> Void
    let onWorkingCopyChange: @Sendable () -> Void
    let isRelevantWorkingCopyChange: @Sendable ([String]) -> Bool

    init(
        repoPath: String,
        onChange: @escaping @Sendable () -> Void,
        onWorkingCopyChange: @escaping @Sendable () -> Void = {},
        isRelevantWorkingCopyChange: @escaping @Sendable ([String]) -> Bool = { _ in true }
    ) {
        self.repoPath = repoPath
        onOpChange = onChange
        self.onWorkingCopyChange = onWorkingCopyChange
        self.isRelevantWorkingCopyChange = isRelevantWorkingCopyChange

        // Operations land in the primary repo; a secondary workspace's .jj/repo is only a pointer to it.
        let primaryRoot = workspacePrimaryRoot(path: repoPath) ?? repoPath
        let opHeads = (primaryRoot as NSString).appendingPathComponent(".jj/repo/op_heads/heads")
        let fileDescriptor = open(opHeads, O_EVTONLY)
        if fileDescriptor >= 0 {
            let src = DispatchSource.makeFileSystemObjectSource(
                fileDescriptor: fileDescriptor,
                eventMask: [.write, .rename, .delete],
                queue: .main
            )
            src.setEventHandler { [weak self] in
                guard let self else { return }
                let now = Date()
                guard now.timeIntervalSince(lastOpFired) > debounceInterval else { return }
                lastOpFired = now
                onOpChange()
            }
            src.setCancelHandler { close(fileDescriptor) }
            src.resume()
            opSource = src
        }

        // 2. Watch working copy
        startWCWatch()
    }

    private func startWCWatch() {
        var context = FSEventStreamContext()
        context.info = Unmanaged.passUnretained(self).toOpaque()

        let paths = [repoPath] as CFArray
        let flags: FSEventStreamCreateFlags =
            UInt32(kFSEventStreamCreateFlagUseCFTypes) |
            UInt32(kFSEventStreamCreateFlagFileEvents) |
            UInt32(kFSEventStreamCreateFlagNoDefer)

        guard let stream = FSEventStreamCreate(
            nil,
            RepoFSWatcher.fsEventCallback,
            &context,
            paths,
            FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
            2.0,
            flags
        ) else { return }

        FSEventStreamSetDispatchQueue(stream, DispatchQueue.global(qos: .utility))
        FSEventStreamStart(stream)
        wcStream = stream
    }

    private static let fsEventCallback: FSEventStreamCallback = { _, info, _, eventPaths, _, _ in
        guard let info else { return }
        let watcher = Unmanaged<RepoFSWatcher>.fromOpaque(info).takeUnretainedValue()
        guard let paths = unsafeBitCast(eventPaths, to: NSArray.self) as? [String] else { return }

        guard watcher.isRelevantWorkingCopyChange(paths) else { return }

        let now = Date()
        guard now.timeIntervalSince(watcher.lastWCFired) > 2.0 else { return }
        watcher.lastWCFired = now
        DispatchQueue.main.async {
            watcher.onWorkingCopyChange()
        }
    }

    deinit {
        opSource?.cancel()
        if let stream = wcStream {
            FSEventStreamStop(stream)
            FSEventStreamInvalidate(stream)
            FSEventStreamRelease(stream)
        }
    }
}
