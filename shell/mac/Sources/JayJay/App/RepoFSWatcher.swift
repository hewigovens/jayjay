import Foundation

/// Watches jj operation heads for changes and triggers a callback.
final class RepoFSWatcher {
    private var source: DispatchSourceFileSystemObject?
    private let debounceInterval: TimeInterval = 0.5
    private var lastFired: Date = .distantPast

    init(repoPath: String, onChange: @escaping @Sendable () -> Void) {
        let opHeads = (repoPath as NSString).appendingPathComponent(".jj/repo/op_heads/heads")
        let fd = open(opHeads, O_EVTONLY)
        guard fd >= 0 else { return }

        let src = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete],
            queue: .main
        )
        src.setEventHandler { [weak self] in
            guard let self else { return }
            let now = Date()
            guard now.timeIntervalSince(self.lastFired) > self.debounceInterval else { return }
            self.lastFired = now
            onChange()
        }
        src.setCancelHandler { close(fd) }
        src.resume()
        self.source = src
    }

    deinit { source?.cancel() }
}
