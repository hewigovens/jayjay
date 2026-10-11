import SwiftUI

extension View {
    /// Yields once so rows added by the same update exist before scrolling to them.
    func revealing(_ path: String?, with proxy: ScrollViewProxy) -> some View {
        task(id: path) {
            guard let path else { return }
            await Task.yield()
            guard !Task.isCancelled else { return }
            proxy.scrollTo(path)
        }
    }
}
