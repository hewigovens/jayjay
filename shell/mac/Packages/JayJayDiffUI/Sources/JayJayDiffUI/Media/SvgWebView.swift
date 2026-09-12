import SwiftUI
import WebKit

struct SvgWebView: NSViewRepresentable {
    let svg: String

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context _: Context) -> WKWebView {
        let webView = WKWebView(frame: .zero, configuration: WKWebViewConfiguration())
        // KVC on private `drawsBackground` — standard transparent WKWebView trick on macOS.
        webView.setValue(false, forKey: "drawsBackground")
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        guard context.coordinator.svg != svg else { return }
        context.coordinator.svg = svg
        webView.loadHTMLString(Self.wrapInHTML(svg), baseURL: nil)
    }

    final class Coordinator {
        var svg: String?
    }

    private static func wrapInHTML(_ svg: String) -> String {
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
                background: transparent; overflow: hidden;
            }
            body > svg { flex-shrink: 0; }
        </style>
        </head>
        <body>
        \(svg)
        <script>
            const svg = document.querySelector('body > svg');
            if (svg) {
                const viewBox = svg.viewBox.baseVal;
                const absoluteLength = name => {
                    const length = svg[name].baseVal;
                    return svg.hasAttribute(name) && length.unitType !== SVGLength.SVG_LENGTHTYPE_PERCENTAGE ? length.value : 0;
                };
                const ratio = viewBox.width > 0 && viewBox.height > 0 ? viewBox.width / viewBox.height : 0;
                let width = absoluteLength('width');
                let height = absoluteLength('height');
                if (!width) width = height && ratio ? height * ratio : viewBox.width || 300;
                if (!height) height = ratio ? width / ratio : viewBox.height || 150;
                const fit = () => {
                    const scale = Math.min(1, document.body.clientWidth / width, document.body.clientHeight / height);
                    svg.style.width = width * scale + 'px';
                    svg.style.height = height * scale + 'px';
                };
                new ResizeObserver(fit).observe(document.body);
                fit();
            }
        </script>
        </body>
        </html>
        """
    }
}
