import Foundation

final class LockedFlag: @unchecked Sendable {
    /// Uninterruptible, like synchronous FFI.
    func setAfterBlocking(seconds: TimeInterval) {
        Thread.sleep(forTimeInterval: seconds)
        set()
    }

    private let lock = NSLock()
    private var value = false

    func set() {
        lock.lock()
        value = true
        lock.unlock()
    }

    var isSet: Bool {
        lock.lock()
        defer { lock.unlock() }
        return value
    }
}
