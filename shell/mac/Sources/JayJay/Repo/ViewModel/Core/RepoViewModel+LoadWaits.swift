import Foundation

@MainActor
extension RepoViewModel {
    /// Follows refreshes that supersede the awaited one; returns early only when one ends without a successor.
    func waitForFirstLoad() async {
        while !hasFinishedFirstLoad, let task = refreshTask {
            await task.value
            guard refreshTask != task else { return }
        }
    }

    func waitForRefresh() async {
        while isRefreshingInFlight, let task = refreshTask {
            await task.value
            guard refreshTask != task else { return }
        }
    }
}
