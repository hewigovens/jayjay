import AppKit
import JayJayCore
import SwiftUI
import WebKit

struct SvgDiffView: View {
    let oldContent: String?
    let newContent: String?
    let hunkType: HunkType

    var body: some View {
        content
            .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private var content: some View {
        switch hunkType {
            case .added:
                pane(svg: newContent, label: "Added", tint: .green)
                    .padding(16)
            case .removed:
                pane(svg: oldContent, label: "Removed", tint: .red)
                    .padding(16)
            case .renamed:
                pane(svg: newContent, label: "Renamed", tint: .blue)
                    .padding(16)
            case .modified:
                HStack(spacing: 12) {
                    pane(svg: oldContent, label: "Before", tint: .red)
                    pane(svg: newContent, label: "After", tint: .green)
                }
                .padding(16)
        }
    }

    @ViewBuilder
    private func pane(svg: String?, label: String, tint: Color) -> some View {
        VStack(spacing: 8) {
            Text(label)
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(tint)
                .padding(.horizontal, 10)
                .padding(.vertical, 3)
                .background(tint.opacity(0.12), in: Capsule())

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
