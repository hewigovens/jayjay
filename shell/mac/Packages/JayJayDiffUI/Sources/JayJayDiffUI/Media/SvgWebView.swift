import Foundation
import SwiftUI
import WebKit

struct SvgWebView: NSViewRepresentable {
    let svg: String

    func makeCoordinator() -> PreviewWebView.Coordinator {
        PreviewWebView.Coordinator()
    }

    func makeNSView(context: Context) -> WKWebView {
        let webView = context.coordinator.makeWebView()
        webView.configuration.userContentController.addUserScript(WKUserScript(
            source: Self.sizingScript, injectionTime: .atDocumentEnd, forMainFrameOnly: true, in: .defaultClient
        ))
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(.renderedHTML(Self.wrapInHTML(svg), location: nil), in: webView)
    }

    private static func wrapInHTML(_ svg: String) -> String {
        let encoded = Data(svg.utf8).base64EncodedString()
        return """
        <!DOCTYPE html>
        <html>
        <head>
        <meta charset="utf-8">
        <meta http-equiv="Content-Security-Policy" content="default-src 'none'; img-src data:; style-src 'unsafe-inline'; base-uri 'none'; form-action 'none'">
        <style>
            html, body {
                margin: 0; padding: 0;
                width: 100%; height: 100%;
                display: flex; align-items: center; justify-content: center;
                background: transparent; overflow: hidden;
            }
            img { flex-shrink: 0; object-fit: contain; max-width: 100%; max-height: 100%; }
        </style>
        </head>
        <body>
        <img src="data:image/svg+xml;base64,\(encoded)">
        </body>
        </html>
        """
    }

    private static let sizingScript = """
    const image = document.querySelector('body > img');
    if (image) {
        const source = new TextDecoder().decode(Uint8Array.from(atob(image.src.split(',')[1]), character => character.charCodeAt(0)));
        const svg = new DOMParser().parseFromString(source, 'image/svg+xml').documentElement;
        if (svg.localName === 'svg' && svg.namespaceURI === 'http://www.w3.org/2000/svg') {
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
                image.style.width = width * scale + 'px';
                image.style.height = height * scale + 'px';
            };
            new ResizeObserver(fit).observe(document.body);
            fit();
        }
    }
    """
}
