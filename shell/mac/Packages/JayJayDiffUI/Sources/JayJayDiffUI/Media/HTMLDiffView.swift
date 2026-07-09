import SwiftUI

/// Loaded through `PreviewWebView`'s scheme handler rooted at the repo checkout, so the markup and any asset it references — including parent-relative ones like `../assets/site.css` — are containment-checked rather than read via `file:`.
public struct HTMLDiffView: View {
    public let location: RepoPreviewLocation

    public init(location: RepoPreviewLocation) {
        self.location = location
    }

    public var body: some View {
        PreviewWebView(source: .file(location))
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(Color(nsColor: .textBackgroundColor))
    }
}
