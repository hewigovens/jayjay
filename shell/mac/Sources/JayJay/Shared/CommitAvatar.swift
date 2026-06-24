import AppKit
import SwiftUI

/// A circular author avatar: the cached image when available, else a stable
/// colored monogram. Reads the in-memory cache synchronously so a known avatar
/// never flashes a placeholder while scrolling.
struct CommitAvatar: View {
    let email: String
    var size: CGFloat = 20
    @State private var image: NSImage?

    init(email: String, size: CGFloat = 20) {
        self.email = email
        self.size = size
        _image = State(initialValue: AvatarStore.cachedImage(email))
    }

    var body: some View {
        content
            .frame(width: size, height: size)
            // Re-seed when a recycled row swaps to a different author.
            .onChange(of: email) { image = AvatarStore.cachedImage(email) }
            .task(id: email) {
                let requested = email
                guard image == nil, !requested.isEmpty else { return }
                let loaded = await AvatarStore.shared.image(for: requested, pixelSize: Int(size * 2))
                // The row may have recycled to a different author mid-fetch.
                guard !Task.isCancelled, email == requested else { return }
                image = loaded
            }
    }

    @ViewBuilder private var content: some View {
        if let image {
            Image(nsImage: image)
                .resizable()
                .aspectRatio(contentMode: .fill)
                .frame(width: size, height: size)
                .clipShape(Circle())
        } else {
            monogram
        }
    }

    private var monogram: some View {
        Circle()
            .fill(Self.monogramColor(email))
            .overlay(
                Text(Self.initial(email))
                    .font(.system(size: size * 0.5, weight: .semibold))
                    .foregroundStyle(.white)
            )
            .frame(width: size, height: size)
    }

    /// Initial from the email's username (after `+` for GitHub noreply addresses).
    private static func initial(_ email: String) -> String {
        let username = AvatarStore.username(from: email)
        let source = username.isEmpty ? email : username
        guard let ch = source.first(where: { $0.isLetter || $0.isNumber }) else { return "?" }
        return String(ch).uppercased()
    }

    /// Deterministic monogram palette, shared with the GPUI shell's `initial_color`.
    static let monogramPalette: [UInt32] = [
        0x4A5568, 0x6B46C1, 0x2563EB, 0x059669, 0xD97706, 0xDC2626, 0xDB2777, 0x0891B2
    ]

    private static func monogramColor(_ email: String) -> Color {
        let byte = UInt8(String(AvatarStore.key(email).prefix(2)), radix: 16) ?? 0
        let hex = monogramPalette[Int(byte) % monogramPalette.count]
        return Color(
            red: Double((hex >> 16) & 0xFF) / 255,
            green: Double((hex >> 8) & 0xFF) / 255,
            blue: Double(hex & 0xFF) / 255
        )
    }
}
