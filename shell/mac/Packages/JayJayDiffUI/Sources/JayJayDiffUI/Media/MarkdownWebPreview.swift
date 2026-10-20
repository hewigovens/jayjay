import JayJayCore
import SwiftUI

struct MarkdownWebPreview: View {
    let markdown: String
    let location: RepoPreviewLocation?
    @State private var source: PreviewWebView.ContentSource?

    var body: some View {
        Group {
            if let source {
                PreviewWebView(source: source)
            } else {
                Color.clear
            }
        }
        .task(id: Request(markdown: markdown, location: location)) {
            let markdown = markdown
            let html = await Task.detached(priority: .userInitiated) {
                renderMarkdownHtml(markdown: markdown)
            }.value
            guard !Task.isCancelled else { return }
            source = .renderedHTML(html, location: location)
        }
    }

    private struct Request: Equatable {
        let markdown: String
        let location: RepoPreviewLocation?
    }
}
