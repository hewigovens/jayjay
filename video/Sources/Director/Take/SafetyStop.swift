import Foundation

/// A take stopped because someone else is using the Mac; the run stops with it instead of launching the next scene in front of them.
struct SafetyStop: Error, CustomStringConvertible {
    let description: String

    init(_ description: String) {
        self.description = description
    }

    private static let lock = NSLock()
    private static var requested = false

    static var isRequested: Bool {
        lock.withLock { requested }
    }

    static func request() {
        lock.withLock { requested = true }
    }

    static func checkRequested() throws {
        if isRequested {
            throw SafetyStop("interrupted")
        }
    }
}
