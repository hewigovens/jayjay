import JayJayCore
import SwiftUI

struct ConvergeVersionValues: View {
    let item: ConvergeItem
    let review: ConvergeReview

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            ForEach(Array(review.versions.enumerated()), id: \.element.commitId) { index, version in
                HStack(alignment: .firstTextBaseline, spacing: 10) {
                    Text(review.letter(of: version.commitId))
                        .jayjayFont(11, weight: .semibold, design: .monospaced)
                        .foregroundStyle(.secondary)
                    let value = (version.value(of: item) ?? "").trimmingCharacters(in: .whitespacesAndNewlines)
                    Text(value.isEmpty ? Localized.Converge.noDescription : value)
                        .jayjayFont(12, design: item == .parents ? .monospaced : .default)
                        .foregroundStyle(value.isEmpty ? .secondary : .primary)
                        .textSelection(.enabled)
                        .fixedSize(horizontal: false, vertical: true)
                    Spacer(minLength: 0)
                }
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(tint(index))
            }
        }
    }

    private func tint(_ index: Int) -> Color {
        guard review.versions.count == 2 else { return .clear }
        return (index == 0 ? Color.red : Color.green).opacity(0.1)
    }
}
