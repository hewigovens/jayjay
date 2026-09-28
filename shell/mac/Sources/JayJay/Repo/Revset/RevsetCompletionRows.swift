import JayJayCore
import SwiftUI

struct RevsetCompletionRows: View {
    static let rowHeight: CGFloat = 28

    let completions: [RevsetCompletion]
    let selected: Int?
    let onPick: (RevsetCompletion) -> Void

    var body: some View {
        ForEach(Array(completions.enumerated()), id: \.offset) { index, completion in
            Button {
                onPick(completion)
            } label: {
                HStack(spacing: 8) {
                    Text(completion.text)
                        .jayjayFont(12, design: .monospaced)
                        .lineLimit(1)
                        .truncationMode(.middle)
                    Text(completion.kind.label)
                        .jayjayFont(11)
                        .foregroundStyle(.secondary)
                    Spacer(minLength: 0)
                }
                .padding(.horizontal, 14)
                .frame(height: Self.rowHeight)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)
            .background(
                RoundedRectangle(cornerRadius: 6)
                    .fill(selected == index ? Color.accentColor.opacity(0.15) : .clear)
                    .padding(.horizontal, 6)
            )
            .id(index)
            .accessibilityIdentifier(AID.Picker.row("completion-\(completion.text)"))
        }
    }
}

extension RevsetCompletion {
    func edit(id: Int) -> FilterFieldEdit {
        FilterFieldEdit(id: id, range: NSRange(location: Int(start), length: Int(len)), text: text)
    }
}

private extension RevsetCompletionKind {
    var label: String {
        switch self {
            case .function: "function"
            case .alias: "alias"
            case .bookmark: "bookmark"
            case .tag: "tag"
        }
    }
}
