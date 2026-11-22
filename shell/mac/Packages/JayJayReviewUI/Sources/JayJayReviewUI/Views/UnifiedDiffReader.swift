import JayJayCore
import SwiftUI

public struct UnifiedDiffReader: View {
    var model: ReviewModel
    var expand: () -> Void
    @Environment(\.colorScheme) private var scheme
    @Environment(\.diffThemes) private var themes
    @State private var userScrolling = false
    @State private var visibleLine: ReaderLine.ID?

    public init(model: ReviewModel, expand: @escaping () -> Void) {
        self.model = model
        self.expand = expand
    }

    public var body: some View {
        let theme = themes.colors(for: scheme)
        Group {
            if model.rendering, model.lines.isEmpty {
                ProgressView(Localized.Web.loadingFile).padding()
            } else if let placeholder = model.placeholder {
                Text(placeholder.text).foregroundStyle(Color(rgb: theme.textDim)).padding()
            } else {
                lineScroll(theme)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
        .background(Color(rgb: theme.contextBg))
    }

    private struct Placement: Equatable {
        let viewport: CGSize
        let path: String?
        let expanded: Bool
        let count: Int
    }

    private func lineScroll(_ theme: DiffThemeColors) -> some View {
        let digits = String(model.lines.reduce(1) { max($0, $1.id.number) }).count
        return GeometryReader { geometry in
            ScrollViewReader { proxy in
                ScrollView {
                    if let notice = model.file?.notice {
                        Text(notice)
                            .font(.footnote)
                            .foregroundStyle(Color(rgb: theme.textDim))
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.horizontal, 12)
                            .padding(.vertical, 6)
                    }
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(model.lines) { line in
                            Group {
                                if line.cell.style == .separator {
                                    fold(line.cell, theme)
                                } else {
                                    row(line.cell, digits: digits, theme)
                                }
                            }
                            .id(line.id)
                        }
                    }
                    .scrollTargetLayout()
                    // The last line must still be able to reach the top when a taller viewport takes over.
                    Color.clear.frame(height: max(0, geometry.size.height - 24)).accessibilityHidden(true)
                }
                .onScrollTargetVisibilityChange(idType: ReaderLine.ID.self, threshold: 1) { ids in
                    visibleLine = model.topmost(of: ids)
                    if userScrolling, let visibleLine {
                        model.anchor = visibleLine
                    }
                }
                .onScrollPhaseChange { old, next in
                    userScrolling = next == .interacting || next == .decelerating
                    if next == .idle, old == .interacting || old == .decelerating, let visibleLine {
                        model.anchor = visibleLine
                    }
                }
                .task(id: Placement(viewport: geometry.size, path: model.path, expanded: model.all, count: model.lines.count)) {
                    // Navigation and resizing lay out passing viewports; scrolling to them must never replace the line the person chose.
                    guard let target = model.anchor else { return }
                    try? await Task.sleep(for: .milliseconds(100))
                    guard !Task.isCancelled, !userScrolling, target == model.anchor else { return }
                    proxy.scrollTo(target, anchor: .top)
                }
            }
        }
    }

    private func fold(_ cell: RenderedDiff.Cell, _ theme: DiffThemeColors) -> some View {
        Button(action: expand) {
            Text(cell.text + " · " + Localized.Diff.showAll)
                .font(.footnote.weight(.semibold))
                .foregroundStyle(Color(rgb: theme.textDim))
                .frame(maxWidth: .infinity, minHeight: 44)
                .background(Color(rgb: theme.separatorBg))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .disabled(model.rendering)
    }

    private func row(_ cell: RenderedDiff.Cell, digits: Int, _ theme: DiffThemeColors) -> some View {
        let (background, marker, sign) = switch cell.style {
            case .added: (theme.addedBg, theme.gutterAddedFg, "+")
            case .removed: (theme.removedBg, theme.gutterRemovedFg, "−")
            default: (theme.contextBg, theme.gutterFg, " ")
        }
        return HStack(alignment: .firstTextBaseline, spacing: 5) {
            Text(String(repeating: "8", count: digits)).hidden().overlay(alignment: .trailing) {
                Text(cell.newLine.map(String.init) ?? cell.oldLine?.text ?? "").foregroundStyle(Color(rgb: theme.gutterFg))
            }
            Text(sign).foregroundStyle(Color(rgb: marker))
            Text(Self.code(cell, theme: theme))
                .fixedSize(horizontal: false, vertical: true)
                .frame(maxWidth: .infinity, alignment: .leading)
        }
        .font(.footnote.monospaced())
        .padding(.horizontal, 12)
        .padding(.vertical, 2)
        .background(Color(rgb: background))
        .accessibilityElement(children: .combine)
    }

    /// Run boundaries are kept exactly; a word change is never inferred from tokens or neighbors.
    static func code(_ cell: RenderedDiff.Cell, theme: DiffThemeColors) -> AttributedString {
        let (text, word): (UInt32, UInt32?) = switch cell.style {
            case .added: (theme.textAdded, theme.addedWordBg)
            case .removed: (theme.textRemoved, theme.removedWordBg)
            default: (theme.textContext, nil)
        }
        var code = AttributedString()
        for span in cell.spans {
            var run = AttributedString(span.text)
            run.foregroundColor = Color(rgb: span.token.flatMap(theme.color(of:)) ?? text)
            if let word, span.style == cell.style {
                run.backgroundColor = Color(rgb: word)
            }
            code.append(run)
        }
        if cell.noEofNewline {
            var note = AttributedString(" ↵ " + Localized.Web.eof)
            note.foregroundColor = Color(rgb: theme.textDim)
            code.append(note)
        }
        return code
    }
}
