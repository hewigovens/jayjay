import ApplicationServices
import Foundation

@main
enum Director {
    static func main() async {
        do {
            switch try Command(Array(CommandLine.arguments.dropFirst())) {
                case let .record(options):
                    try await record(options)
                case let .inspect(app, pid, includeMenuBar):
                    try requireAccessibility()
                    let pid = try pid ?? AppSession.runningPID(of: URL(fileURLWithPath: app))
                    TreeDump.print(AXElement.application(pid), includeMenuBar: includeMenuBar)
                case let .web(clip):
                    try await clip.write()
                case .help:
                    print(Command.usage)
            }
        } catch {
            FileHandle.standardError.write(Data("director: \(error)\n".utf8))
            exit(1)
        }
    }

    private static func record(_ options: RunOptions) async throws {
        try requireAccessibility()
        let scenes = try Scene.load(options.scenes.map { URL(fileURLWithPath: $0) })
        try FileManager.default.createDirectory(atPath: options.out, withIntermediateDirectories: true)
        let interrupts = handleInterrupts(quittingApp: !options.keepOpen)
        defer { interrupts.forEach { $0.cancel() } }
        var failures = 0
        for scene in scenes {
            try SafetyStop.checkRequested()
            do {
                try await Take(scene: scene, options: options).record()
                print("\(scene.name): recorded")
            } catch let stop as SafetyStop {
                throw Failure("\(scene.name): \(stop); the remaining scenes were not recorded")
            } catch {
                print("\(scene.name): failed: \(error)")
                failures += 1
            }
        }
        if failures > 0 {
            throw Failure("\(failures) of \(scenes.count) scenes failed")
        }
    }

    private static func handleInterrupts(quittingApp: Bool) -> [DispatchSourceSignal] {
        [SIGINT, SIGTERM].map { number in
            signal(number, SIG_IGN)
            let source = DispatchSource.makeSignalSource(signal: number, queue: .global())
            // The first signal lets the take finish its clip; a second quits at once.
            source.setEventHandler {
                if SafetyStop.isRequested {
                    TakeCleanup.shared.run(quittingApp: quittingApp)
                    exit(128 + number)
                }
                SafetyStop.request()
                print("director: stopping after this step; interrupt again to quit now")
            }
            source.resume()
            return source
        }
    }

    private static func requireAccessibility() throws {
        guard AXIsProcessTrusted() else {
            throw Failure("allow Accessibility for the terminal that runs record in System Settings › Privacy & Security › Accessibility")
        }
    }
}
