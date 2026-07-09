import Foundation
import JayJayCore
import SwiftUI

public struct MarkdownDiffView: View {
    public let markdown: String?
    private let baseURL: URL?
    private let renderPlan: MarkdownRenderPlan

    public init(markdown: String?, baseURL: URL? = nil) {
        self.markdown = markdown
        self.baseURL = baseURL
        renderPlan = Self.renderPlan(for: markdown)
    }

    public var body: some View {
        Group {
            switch renderPlan {
                case .empty:
                    Text("No post-change Markdown content.")
                        .foregroundStyle(.secondary)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                case let .web(markdown):
                    MarkdownWebView(html: renderMarkdownHtml(markdown: markdown), baseURL: baseURL)
                case let .plainPreview(preview, truncated):
                    ScrollView {
                        VStack(alignment: .leading, spacing: 12) {
                            if truncated {
                                Text("Large Markdown preview truncated to keep JayJay responsive.")
                                    .font(.callout)
                                    .foregroundStyle(.secondary)
                            }
                            Text(preview)
                                .font(.system(.body, design: .monospaced))
                                .textSelection(.enabled)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                        .padding(18)
                    }
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(nsColor: .textBackgroundColor))
    }

    static let richMarkdownByteLimit = 256 * 1024
    static let largePlainPreviewCharacterLimit = 80000

    static func renderPlan(for markdown: String?) -> MarkdownRenderPlan {
        guard let markdown, !markdown.isEmpty else { return .empty }
        guard markdown.utf8.count > richMarkdownByteLimit else {
            return .web(markdown)
        }

        let prefix = markdown.prefix(largePlainPreviewCharacterLimit + 1)
        if prefix.count > largePlainPreviewCharacterLimit {
            return .plainPreview(
                String(prefix.dropLast()),
                truncated: true
            )
        }
        return .plainPreview(String(prefix), truncated: false)
    }
}

enum MarkdownRenderPlan: Equatable {
    case empty
    case web(String)
    case plainPreview(String, truncated: Bool)
}
