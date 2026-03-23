import Foundation

/// Watches jj operation heads and working copy for changes.
final class RepoFSWatcher {
    private var opSource: DispatchSourceFileSystemObject?
    private var wcStream: FSEventStreamRef?
    private let debounceInterval: TimeInterval = 1.0
    private var lastOpFired: Date = .distantPast
    private var lastWCFired: Date = .distantPast
    let repoPath: String
    private let ignoredPrefixes: [String]

    let onOpChange: @Sendable () -> Void
    let onWorkingCopyChange: @Sendable () -> Void

    init(
        repoPath: String,
        onChange: @escaping @Sendable () -> Void,
        onWorkingCopyChange: @escaping @Sendable () -> Void = {}
    ) {
        self.repoPath = repoPath
        onOpChange = onChange
        self.onWorkingCopyChange = onWorkingCopyChange
        ignoredPrefixes = Self.loadIgnoredPrefixes(repoPath: repoPath)

        // 1. Watch jj op_heads (triggers auto-refresh)
        let opHeads = (repoPath as NSString).appendingPathComponent(".jj/repo/op_heads/heads")
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

    /// Parse .gitignore for directory prefixes to ignore, plus always .jj/ .git/
    private static func loadIgnoredPrefixes(repoPath: String) -> [String] {
        var prefixes = [".jj/", ".git/", ".DS_Store"]
        let gitignorePath = (repoPath as NSString).appendingPathComponent(".gitignore")
        if let contents = try? String(contentsOfFile: gitignorePath, encoding: .utf8) {
            for line in contents.components(separatedBy: .newlines) {
                let trimmed = line.trimmingCharacters(in: .whitespaces)
                if trimmed.isEmpty || trimmed.hasPrefix("#") { continue }
                // "target/" or "build/" style patterns → use as prefix
                let clean = trimmed.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
                if !clean.isEmpty {
                    prefixes.append(clean + "/")
                    prefixes.append(clean) // also match the dir name itself
                }
            }
        }
        return prefixes
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

        let prefix = watcher.repoPath.hasSuffix("/") ? watcher.repoPath : watcher.repoPath + "/"
        let hasRelevant = paths.contains { path in
            let relative = path.hasPrefix(prefix) ? String(path.dropFirst(prefix.count)) : path
            return !watcher.ignoredPrefixes.contains(where: { relative.hasPrefix($0) })
        }
        guard hasRelevant else { return }

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
