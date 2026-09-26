import SwiftUI

struct OverviewChipRow: View {
    struct Chip: Hashable {
        let text: String
        let tint: Color
    }

    let chips: [Chip]

    var body: some View {
        ViewThatFits(in: .horizontal) {
            ForEach(Array(stride(from: chips.count, to: 0, by: -1)), id: \.self) { shown in
                row(shown: shown, fixed: true)
            }
            row(shown: min(chips.count, 1), fixed: false)
        }
    }

    private func row(shown: Int, fixed: Bool) -> some View {
        HStack(spacing: 4) {
            ForEach(chips.prefix(shown), id: \.self) { chip in
                OverviewChip(text: chip.text, tint: chip.tint)
                    .fixedSize(horizontal: fixed, vertical: false)
            }
            if chips.count > shown {
                OverviewChip(text: "+\(chips.count - shown)", tint: .secondary)
                    .fixedSize()
            }
            Spacer(minLength: 0)
        }
    }
}

struct OverviewChip: View {
    let text: String
    let tint: Color

    var body: some View {
        Text(text)
            .jayjayFont(9, weight: .semibold, design: .monospaced)
            .lineLimit(1)
            .truncationMode(.middle)
            .padding(.horizontal, 5)
            .padding(.vertical, 1)
            .background(tint.opacity(0.16), in: RoundedRectangle(cornerRadius: 4))
            .foregroundStyle(tint)
    }
}
