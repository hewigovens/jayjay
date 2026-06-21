import JayJayCore
import JayJayDiffUI
import SwiftUI

struct AnnotateView: View {
    let lines: [AnnotationLine]
    let path: String
    let onSelectChange: (String) -> Void
    let onDismiss: () -> Void

    /// Per-line syntax spans, computed once from the file content (no repo needed).
    @State private var highlightedSpans: [[DiffSpan]]?
    @Environment(AppSettings.self) private var settings
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            GeometryReader { geo in
                ScrollView([.horizontal, .vertical]) {
                    VStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(lines.enumerated()), id: \.offset) { idx, line in
                            HStack(alignment: .top, spacing: 0) {
                                gutterView(line: line)
                                highlightedText(index: idx, fallback: line.text)
                                    .textSelection(.enabled)
                            }
                            .padding(.vertical, 1)
                        }
                    }
                    .padding(12)
                    .frame(minWidth: geo.size.width, minHeight: geo.size.height, alignment: .topLeading)
                }
            }
        }
        // Re-run when lines arrive: annotate loads them async after this view first appears.
        .task(id: lines.count) { await computeHighlights() }
    }

    private var header: some View {
        HStack {
            Image(systemName: "text.line.first.and.arrowtriangle.forward")
                .foregroundStyle(.secondary)
            Text("Annotate: \(path)")
                .jayjayFont(13, weight: .semibold, design: .monospaced)
                .lineLimit(1)
            Spacer()
            Text("\(lines.count) lines")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            Button("Exit Annotate", action: onDismiss)
                .keyboardShortcut(.cancelAction)
                .help("Close annotate view (esc)")
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
    }

    private func gutterView(line: AnnotationLine) -> some View {
        HStack(spacing: 6) {
            Text(String(line.lineNumber))
                .frame(width: 36, alignment: .trailing)
                .foregroundStyle(.tertiary)
            changeIdText(line.changeId)
                .help("Click to select this change")
                .onTapGesture { onSelectChange(line.changeId.id) }
            Text(line.author.prefix(8))
                .foregroundStyle(.secondary)
                .frame(width: 64, alignment: .leading)
                .lineLimit(1)
            Text(dateLabel(line.timestamp))
                .foregroundStyle(.tertiary)
                .frame(width: 80, alignment: .trailing)
        }
        .jayjayFont(settings.fontSize, design: .monospaced)
        .padding(.trailing, 12)
    }

    /// Blame change-id: per-change color (so lines group visually) with the
    /// shortest-unique prefix at full strength and the remainder dimmed.
    private func changeIdText(_ shortId: ShortId) -> Text {
        let color = changeColor(shortId.id)
        let shown = String(shortId.id.prefix(8))
        let n = max(0, min(Int(shortId.shortLen), shown.count))
        let split = shown.index(shown.startIndex, offsetBy: n)
        var attr = AttributedString(shown[..<split])
        attr.foregroundColor = color
        var rest = AttributedString(shown[split...])
        rest.foregroundColor = color.opacity(0.5)
        attr.append(rest)
        return Text(attr)
    }

    // MARK: - Syntax highlighting

    private var monoFont: Font {
        .system(size: settings.fontSize, design: .monospaced)
    }

    @ViewBuilder
    private func highlightedText(index: Int, fallback: String) -> some View {
        if let spans = highlightedSpans, index < spans.count {
            let colors = DiffColors(isDark: colorScheme == .dark)
            let line = spans[index].reduce(into: AttributedString()) { result, span in
                let color = Color(nsColor: colors.tokenColor(span.token, fallback: colors.contextText))
                var s = AttributedString(span.text)
                s.foregroundColor = color
                result.append(s)
            }
            Text(line).font(monoFont)
        } else {
            Text(fallback)
                .font(monoFont)
        }
    }

    private func computeHighlights() async {
        let fullText = lines.map(\.text).joined(separator: "\n")
        let p = path
        highlightedSpans = await Task.detached {
            highlightFileLines(path: p, content: fullText)
        }.value
    }

    // MARK: - Helpers

    private func changeColor(_ changeId: String) -> Color {
        let hash = changeId.unicodeScalars.reduce(0) { $0 &+ Int($1.value) }
        let hue = Double(hash % 360) / 360.0
        return Color(hue: hue, saturation: 0.5, brightness: 0.7)
    }

    /// Blame dates stay absolute (yyyy-MM-dd) so they line up and scan cleanly,
    /// matching the GPUI shell — no relative "20d ago".
    private func dateLabel(_ timestamp: String) -> String {
        String(timestamp.prefix(10))
    }
}
