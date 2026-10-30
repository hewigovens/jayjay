import SwiftUI

struct FlockIconPicker: View {
    @Environment(FlockModel.self) private var flock
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        LazyVGrid(columns: [GridItem(.adaptive(minimum: 88), spacing: 12, alignment: .top)], spacing: 12) {
            ForEach(FlockIcon.allCases) { icon in
                Button {
                    flock.choose(icon)
                } label: {
                    VStack(spacing: 6) {
                        Image(nsImage: icon.image(dark: colorScheme == .dark))
                            .resizable()
                            .interpolation(.high)
                            .frame(width: 72, height: 72)
                            .overlay(alignment: .topTrailing) {
                                if flock.activeIcon == icon {
                                    Image(systemName: "checkmark.circle.fill")
                                        .foregroundStyle(.white, Color.accentColor)
                                }
                            }
                        Text(icon.title)
                            .jayjayFont(11)
                            .multilineTextAlignment(.center)
                    }
                    .padding(6)
                    .frame(maxWidth: .infinity)
                    .background(flock.activeIcon == icon ? Color.accentColor.opacity(0.12) : .clear, in: RoundedRectangle(cornerRadius: 12))
                }
                .buttonStyle(.plain)
                .accessibilityLabel(icon.title)
                .accessibilityAddTraits(flock.activeIcon == icon ? .isSelected : [])
                .accessibilityIdentifier(AID.Settings.flockIcon(icon.rawValue))
            }
        }
    }
}
