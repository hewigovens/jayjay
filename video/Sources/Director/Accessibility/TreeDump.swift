import ApplicationServices
import Foundation

enum TreeDump {
    static func print(_ app: AXElement, includeMenuBar: Bool) {
        var stack = [(element: app, depth: 0)]
        while let (element, depth) = stack.popLast() {
            let summary = element.summary()
            if summary.role == kAXMenuBarRole, !includeMenuBar {
                continue
            }
            if let line = describe(element, summary) {
                Swift.print(String(repeating: "  ", count: depth) + line)
            }
            stack.append(contentsOf: summary.children.reversed().map { ($0, depth + 1) })
        }
    }

    private static func describe(_ element: AXElement, _ summary: AXElement.Summary) -> String? {
        let value = element.string(kAXValueAttribute).map { $0.count > 60 ? $0.prefix(60) + "…" : $0 }?.replacingOccurrences(of: "\n", with: "⏎")
        let texts = [summary.title, summary.description, value].compactMap { $0 }.filter { !$0.isEmpty }
        guard summary.identifier != nil || !texts.isEmpty else { return nil }
        var parts = [summary.role ?? "?"]
        if let identifier = summary.identifier {
            parts.append("id=\(identifier)")
        }
        parts += texts.map { "\"\($0)\"" }
        if let frame = element.frame {
            parts.append("@\(Int(frame.minX)),\(Int(frame.minY)) \(Int(frame.width))x\(Int(frame.height))")
        }
        return parts.joined(separator: " ")
    }
}
