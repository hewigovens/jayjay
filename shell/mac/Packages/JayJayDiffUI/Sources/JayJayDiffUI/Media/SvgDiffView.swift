import AppKit
import JayJayCore
import SwiftUI
import WebKit

public struct SvgDiffView: View {
    public let oldContent: String?
    public let newContent: String?
    public let hunkType: HunkType

    public init(oldContent: String?, newContent: String?, hunkType: HunkType) {
        self.oldContent = oldContent
        self.newContent = newContent
        self.hunkType = hunkType
    }

    public var body: some View {
        content
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private var content: some View {
        switch hunkType {
            case .added:
                pane(svg: newContent, label: "Added", tint: .green, showsLabel: false)
                    .padding(16)
            case .removed:
                pane(svg: oldContent, label: "Removed", tint: .red, showsLabel: false)
                    .padding(16)
            case .renamed:
                pane(svg: newContent, label: "Renamed", tint: .blue, showsLabel: false)
                    .padding(16)
            case .modified:
                HStack(spacing: 12) {
                    pane(svg: oldContent, label: "Before", tint: .red)
                    pane(svg: newContent, label: "After", tint: .green)
                }
                .padding(16)
        }
    }

    private func pane(
        svg: String?,
        label: String,
        tint: Color,
        showsLabel: Bool = true
    ) -> some View {
        VStack(spacing: 8) {
            if showsLabel {
                Text(label)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(tint)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 3)
                    .background(tint.opacity(0.12), in: Capsule())
            }

            Group {
                if let svg, !svg.isEmpty {
                    SvgWebView(svg: svg)
                } else {
                    Text("—")
                        .foregroundStyle(.secondary)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(Color.primary.opacity(0.12), lineWidth: 1)
            )
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

private struct SvgWebView: NSViewRepresentable {
    let svg: String

    func makeNSView(context _: Context) -> WKWebView {
        let webView = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        // KVC on private `drawsBackground` — standard transparent WKWebView trick on macOS.
        webView.setValue(false, forKey: "drawsBackground")
        webView.loadHTMLString(wrapInHTML(svg), baseURL: nil)
        return webView
    }

    func updateNSView(_ webView: WKWebView, context _: Context) {
        webView.loadHTMLString(wrapInHTML(svg), baseURL: nil)
    }

    private func wrapInHTML(_ svg: String) -> String {
        """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <style>
            html, body {
                margin: 0; padding: 0;
                width: 100%; height: 100%;
                display: flex; align-items: center; justify-content: center;
                background: transparent;
            }
            svg { max-width: 100%; max-height: 100%; }
        </style>
        </head>
        <body>
        \(svg)
        </body>
        </html>
        """
    }
}
