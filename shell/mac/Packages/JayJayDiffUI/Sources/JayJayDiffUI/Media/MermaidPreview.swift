import WebKit

/// Uses an isolated content world to keep page JavaScript disabled.
@MainActor
enum MermaidPreview {
    nonisolated static let blockMarker = "<code class=\"language-mermaid\">"

    private static let userScript: WKUserScript? = {
        let sources = ["pele.global", "mount"].map { name in
            Bundle.module.url(forResource: name, withExtension: "js", subdirectory: "Mermaid")
                .flatMap { try? String(contentsOf: $0, encoding: .utf8) }
        }
        guard let pele = sources[0], let mount = sources[1] else { return nil }
        return WKUserScript(
            source: pele + "\n" + mount,
            injectionTime: .atDocumentEnd,
            forMainFrameOnly: true,
            in: .world(name: "JayJayMermaid")
        )
    }()

    static func install(in controller: WKUserContentController, enabled: Bool) {
        controller.removeAllUserScripts()
        if enabled, let userScript {
            controller.addUserScript(userScript)
        }
    }
}
