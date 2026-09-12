import Foundation

@MainActor
@Observable
final class SettingsSnapshot<Value: Sendable> {
    private(set) var value: Value?
    @ObservationIgnored private var task: Task<Value, Never>?

    func load(_ read: @escaping @Sendable () -> Value) async {
        guard value == nil else { return }
        let pending = task ?? Task.detached(priority: .utility, operation: read)
        task = pending
        let result = await pending.value
        // Retain the pending read when a tab disappears so reopening it does not launch another process.
        guard !Task.isCancelled else { return }
        value = result
        task = nil
    }
}
