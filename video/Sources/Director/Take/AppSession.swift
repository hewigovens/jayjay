import AppKit
import ApplicationServices

final class AppSession {
    let pid: pid_t
    let app: AXElement

    private init(pid: pid_t) {
        self.pid = pid
        app = AXElement.application(pid)
    }

    /// `-g` leaves focus alone until the take activates the app.
    static func launch(bundle: URL, scene: Scene, log: URL) async throws -> AppSession {
        let executable = try executablePath(of: bundle)
        let previous = pids(running: executable)
        for pid in previous {
            await quit(pid)
        }
        var arguments = ["-g", "-n", bundle.path, "--stdout", "/dev/null", "--stderr", log.path]
        for (key, value) in scene.env.sorted(by: { $0.key < $1.key }) {
            arguments += ["--env", "\(key)=\(value)"]
        }
        arguments.append("--args")
        if let repo = scene.repo {
            arguments += ["--repo", URL(fileURLWithPath: repo).standardizedFileURL.path]
        }
        for (key, value) in scene.defaults.sorted(by: { $0.key < $1.key }) {
            arguments += ["-\(key)", value.replacingWindow(with: scene.window).argument]
        }
        try Shell.run("/usr/bin/open", arguments)
        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline {
            if let pid = pids(running: executable).first(where: { !previous.contains($0) }) {
                return AppSession(pid: pid)
            }
            try await Task.sleep(for: .milliseconds(100))
        }
        throw Failure("\(bundle.lastPathComponent) did not start; see \(log.path)")
    }

    static func runningPID(of bundle: URL) throws -> pid_t {
        guard let pid = try pids(running: executablePath(of: bundle)).last else {
            throw Failure("\(bundle.path) is not running; start a scene with `director record --keep-open`")
        }
        return pid
    }

    static func quit(_ pid: pid_t) async {
        NSRunningApplication(processIdentifier: pid)?.terminate()
        let deadline = Date().addingTimeInterval(5)
        while kill(pid, 0) == 0, Date() < deadline {
            try? await Task.sleep(for: .milliseconds(100))
        }
        guard kill(pid, 0) == 0 else { return }
        NSRunningApplication(processIdentifier: pid)?.forceTerminate()
        let killDeadline = Date().addingTimeInterval(3)
        while kill(pid, 0) == 0, Date() < killDeadline {
            try? await Task.sleep(for: .milliseconds(100))
        }
    }

    func waitForWindow() async throws -> CGRect {
        var previous: CGRect?
        var steady = 0
        let deadline = Date().addingTimeInterval(60)
        while Date() < deadline {
            let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
            let frames = windows.compactMap { info -> CGRect? in
                guard info[kCGWindowOwnerPID as String] as? pid_t == pid, info[kCGWindowLayer as String] as? Int == 0,
                      let bounds = info[kCGWindowBounds as String] as? NSDictionary
                else { return nil }
                return CGRect(dictionaryRepresentation: bounds)
            }
            if let frame = frames.max(by: { $0.width * $0.height < $1.width * $1.height }), frame.width >= 400 {
                steady = frame == previous ? steady + 1 : 0
                previous = frame
                if steady >= 3 {
                    return frame
                }
            }
            try await Task.sleep(for: .milliseconds(150))
        }
        throw Failure("the app showed no window within 60 s")
    }

    func place(_ frame: CGRect) throws {
        guard let mainWindow else { throw Failure("the app has no window to place") }
        mainWindow.setFrame(frame)
    }

    func activate() async throws {
        let deadline = Date().addingTimeInterval(3)
        while Date() < deadline {
            app.set(kAXFrontmostAttribute, to: kCFBooleanTrue)
            mainWindow?.perform(kAXRaiseAction)
            try await Task.sleep(for: .milliseconds(100))
            if hasFocus {
                return
            }
        }
        throw Failure("the app did not come to the front")
    }

    var hasFocus: Bool {
        AXElement.focusedApplicationPID == pid
    }

    func coveringApp(at point: CGPoint, over target: AXElement) -> String? {
        let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly], kCGNullWindowID) as? [[String: Any]] ?? []
        let displays = Self.displayFrames
        for info in windows {
            guard info[kCGWindowAlpha as String] as? Double ?? 1 > 0,
                  let bounds = info[kCGWindowBounds as String] as? NSDictionary,
                  let frame = CGRect(dictionaryRepresentation: bounds), frame.contains(point)
            else { continue }
            let owner = info[kCGWindowOwnerPID as String] as? pid_t
            // Screen-sized overlays such as the Dock's pass clicks through.
            if owner != pid, info[kCGWindowLayer as String] as? Int ?? 0 > 0, displays.contains(frame) {
                continue
            }
            guard owner == pid else { return info[kCGWindowOwnerName as String] as? String ?? "another app" }
            // Overlays that take clicks, such as Mission Control, own the hit element.
            guard let hit = AXElement.element(at: point) else { return nil }
            if let hitPID = hit.pid, hitPID != pid {
                return NSRunningApplication(processIdentifier: hitPID)?.localizedName ?? "another app"
            }
            guard !hit.isInside(target), hit.topLevel != target.topLevel else { return nil }
            return "a sheet or another window of the app"
        }
        return "the desktop"
    }

    private static var displayFrames: [CGRect] {
        var count: UInt32 = 0
        CGGetActiveDisplayList(0, nil, &count)
        var displays = [CGDirectDisplayID](repeating: 0, count: Int(count))
        CGGetActiveDisplayList(count, &displays, &count)
        return displays.map(CGDisplayBounds)
    }

    var mainWindow: AXElement? {
        app.windows.max { ($0.frame?.width ?? 0) < ($1.frame?.width ?? 0) }
    }

    private static func executablePath(of bundle: URL) throws -> String {
        guard let path = Bundle(url: bundle)?.executablePath else {
            throw Failure("\(bundle.path) is not an app bundle; build it with `just shell::build`")
        }
        return path
    }

    private static func pids(running executable: String) -> [pid_t] {
        let executable = realPath(executable) ?? executable
        var pids = [pid_t](repeating: 0, count: Int(proc_listallpids(nil, 0)) + 64)
        let count = Int(proc_listallpids(&pids, Int32(pids.count * MemoryLayout<pid_t>.size)))
        var path = [CChar](repeating: 0, count: 4 * Int(MAXPATHLEN))
        return pids.prefix(max(count, 0)).filter { pid in
            proc_pidpath(pid, &path, UInt32(path.count)) > 0 && realPath(String(cString: path)) == executable
        }.sorted()
    }

    /// Foundation's symlink resolution drops `/private`, which the kernel reports for `/tmp` and `/var`.
    static func realPath(_ path: String) -> String? {
        guard let resolved = realpath(path, nil) else { return nil }
        defer { free(resolved) }
        return String(cString: resolved)
    }
}
