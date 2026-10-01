import Foundation

enum Command {
    case record(RunOptions)
    case inspect(app: String, pid: pid_t?, includeMenuBar: Bool)
    case web(WebClip)
    case help

    static let usage = """
    Usage:
      director record [--app <JayJay.app>] [--out <dir>] [--keep-open] [--no-capture] <scene.json>...
      director inspect [--app <JayJay.app>] [--pid <pid>] [--menu-bar]
      director web [--from <seconds>] [--out <dir>] <take.mov>

    record   Builds each scene's fixture, launches the app, plays the steps, and writes
             <out>/<scene>.mov, <scene>.cues.jsonl, and <scene>.log.
    inspect  Prints the running app's accessibility tree so a scene can name its targets.
    web      Writes <out>/<take>.mp4 in H.264 from --from on, <take>.png of that frame, and prints
             the take's click cues on the clip's clock.
    """

    init(_ arguments: [String]) throws {
        var rest = arguments[...]
        switch rest.popFirst() {
            case "record":
                var options = RunOptions()
                while let argument = rest.popFirst() {
                    switch argument {
                        case "--app": options.app = try Self.value(of: argument, in: &rest)
                        case "--out": options.out = try Self.value(of: argument, in: &rest)
                        case "--keep-open": options.keepOpen = true
                        case "--no-capture": options.capture = false
                        case let option where option.hasPrefix("--"): throw Failure("unknown option \(option)")
                        default: options.scenes.append(argument)
                    }
                }
                guard !options.scenes.isEmpty else { throw Failure("name at least one scene file\n\n\(Self.usage)") }
                self = .record(options)
            case "inspect":
                var app = RunOptions().app
                var pid: pid_t?
                var includeMenuBar = false
                while let argument = rest.popFirst() {
                    switch argument {
                        case "--app": app = try Self.value(of: argument, in: &rest)
                        case "--pid":
                            let value = try Self.value(of: argument, in: &rest)
                            guard let number = pid_t(value) else { throw Failure("--pid needs a number, got \(value)") }
                            pid = number
                        case "--menu-bar": includeMenuBar = true
                        default: throw Failure("unknown option \(argument)")
                    }
                }
                self = .inspect(app: app, pid: pid, includeMenuBar: includeMenuBar)
            case "web":
                var from = 0.0
                var out = "build/recordings/web"
                var take: String?
                while let argument = rest.popFirst() {
                    switch argument {
                        case "--from":
                            let value = try Self.value(of: argument, in: &rest)
                            guard let seconds = Double(value), seconds >= 0 else { throw Failure("--from needs seconds, got \(value)") }
                            from = seconds
                        case "--out": out = try Self.value(of: argument, in: &rest)
                        case let option where option.hasPrefix("--"): throw Failure("unknown option \(option)")
                        default:
                            guard take == nil else { throw Failure("name one take") }
                            take = argument
                    }
                }
                guard let take else { throw Failure("name a take\n\n\(Self.usage)") }
                self = .web(WebClip(take: URL(fileURLWithPath: take), from: from, out: URL(fileURLWithPath: out)))
            case nil, "help", "--help", "-h":
                self = .help
            case let verb?:
                throw Failure("unknown command \(verb)\n\n\(Self.usage)")
        }
    }

    private static func value(of option: String, in rest: inout ArraySlice<String>) throws -> String {
        guard let value = rest.popFirst() else { throw Failure("\(option) needs a value") }
        return value
    }
}
