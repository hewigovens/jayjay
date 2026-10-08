import CoreServices
import Foundation

/// A recursive FSEvents stream on one directory. FSEvents owns the handler, so callbacks still queued when the subscription is dropped never reach freed memory.
final class FSEventSubscription {
    private let stream: FSEventStreamRef

    init?(path: String, latency: CFTimeInterval, handler: @escaping @Sendable ([String]) -> Void) {
        let box = Handler(handler)
        var context = FSEventStreamContext(
            version: 0,
            info: Unmanaged.passUnretained(box).toOpaque(),
            retain: { info in
                info.map { UnsafeRawPointer(Unmanaged<Handler>.fromOpaque($0).retain().toOpaque()) }
            },
            release: { info in
                info.map { Unmanaged<Handler>.fromOpaque($0).release() }
            },
            copyDescription: nil
        )
        let flags = FSEventStreamCreateFlags(
            kFSEventStreamCreateFlagUseCFTypes | kFSEventStreamCreateFlagFileEvents | kFSEventStreamCreateFlagNoDefer
        )
        let stream = withExtendedLifetime(box) {
            FSEventStreamCreate(
                nil,
                Self.callback,
                &context,
                [path] as CFArray,
                FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
                latency,
                flags
            )
        }
        guard let stream else { return nil }

        self.stream = stream
        FSEventStreamSetDispatchQueue(stream, DispatchQueue.global(qos: .utility))
        FSEventStreamStart(stream)
    }

    deinit {
        FSEventStreamStop(stream)
        FSEventStreamInvalidate(stream)
        FSEventStreamRelease(stream)
    }

    private static let callback: FSEventStreamCallback = { _, info, _, eventPaths, _, _ in
        guard let info, let paths = unsafeBitCast(eventPaths, to: NSArray.self) as? [String] else { return }
        Unmanaged<Handler>.fromOpaque(info).takeUnretainedValue().handle(paths)
    }
}

private final class Handler {
    let handle: @Sendable ([String]) -> Void

    init(_ handle: @escaping @Sendable ([String]) -> Void) {
        self.handle = handle
    }
}
