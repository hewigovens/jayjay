import SwiftUI
import Textual

public struct MarkdownDiffView: View {
    public let markdown: String?
    private let renderPlan: MarkdownRenderPlan

    public init(markdown: String?) {
        self.markdown = markdown
        self.renderPlan = Self.renderPlan(for: markdown)
    }

    public var body: some View {
        Group {
            switch renderPlan {
            case .empty:
                Text("No post-change Markdown content.")
                    .foregroundStyle(.secondary)
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            case let .rich(markdown):
                ScrollView {
                    StructuredText(markdown: markdown)
                        .textual.structuredTextStyle(.gitHub)
                        .textual.textSelection(.enabled)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(18)
                }
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
    static let largePlainPreviewCharacterLimit = 80_000

    static func renderPlan(for markdown: String?) -> MarkdownRenderPlan {
        guard let markdown, !markdown.isEmpty else { return .empty }
        guard markdown.utf8.count > richMarkdownByteLimit else {
            return .rich(markdown)
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
    case rich(String)
    case plainPreview(String, truncated: Bool)
}
