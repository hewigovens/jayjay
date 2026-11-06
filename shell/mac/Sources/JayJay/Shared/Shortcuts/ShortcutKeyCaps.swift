import SwiftUI

struct ShortcutKeyCaps: View {
    let chords: [[String]]

    var body: some View {
        HStack(spacing: 3) {
            ForEach(Array(chords.enumerated()), id: \.offset) { index, keys in
                if index > 0 { Text("/").foregroundStyle(.secondary) }
                ForEach(Array(keys.enumerated()), id: \.offset) { _, key in
                    Text(key)
                        .jayjayFont(12, weight: .medium, design: .monospaced)
                        .frame(minWidth: 16)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 3)
                        .background(Color.primary.opacity(0.07), in: RoundedRectangle(cornerRadius: 4))
                        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.primary.opacity(0.12), lineWidth: 0.5))
                }
            }
        }
    }
}
