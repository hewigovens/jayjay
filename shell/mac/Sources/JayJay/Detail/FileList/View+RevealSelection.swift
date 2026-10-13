import SwiftUI

extension View {
    /// Yields once so rows added by the same update exist before scrolling to them.
    func revealing(_ path: String?, again trigger: UUID?, with proxy: ScrollViewProxy) -> some View {
        task(id: [path, trigger?.uuidString]) {
            guard let path else { return }
            await Task.yield()
            guard !Task.isCancelled else { return }
            proxy.scrollTo(path)
        }
    }
}
