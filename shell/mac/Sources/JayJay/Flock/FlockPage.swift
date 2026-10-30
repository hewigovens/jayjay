import AppKit
import JayJayCore
import SwiftUI

struct FlockPage: View {
    @Environment(FlockModel.self) private var flock
    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var flying = false
    @State private var forgetError: String?
    private static let featherImage = Bundle.main.url(forResource: "flock-feather", withExtension: "png")
        .flatMap { NSImage(contentsOf: $0) }

    var body: some View {
        ZStack {
            ScrollView { content }
                .blur(radius: flock.isUnlocked ? 0 : 6)
                .disabled(!flock.isUnlocked)
                .accessibilityHidden(!flock.isUnlocked)
            if !flock.isUnlocked {
                VStack(spacing: 12) {
                    Image(systemName: "gift.fill")
                        .font(.system(size: 24))
                        .foregroundStyle(Color.accentColor)
                        .accessibilityHidden(true)
                    Text(Localized.Settings.flockClaim).jayjayFont(19, weight: .semibold)
                    Text(Localized.Settings.flockVerifyDetail)
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                    FlockCodeButton()
                        .buttonStyle(.borderedProminent)
                    FlockUnlockButton(title: Localized.Settings.flockUnlock)
                        .buttonStyle(.link)
                        .jayjayFont(11)
                }
                .multilineTextAlignment(.center)
                .padding(24)
                .frame(maxWidth: 360)
                .background(.regularMaterial, in: RoundedRectangle(cornerRadius: 16))
                .padding(24)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .ignoresSafeArea(.container, edges: .top)
            }
        }
        .jayjayFont(12)
        .onAppear { replay() }
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Settings.flockContent)
    }

    private var content: some View {
        VStack(spacing: 20) {
            VStack(alignment: .leading, spacing: 6) {
                HStack(alignment: .firstTextBaseline) {
                    Text(Localized.Settings.flockTitle).jayjayFont(24, weight: .bold)
                    Spacer()
                    if flock.member != nil {
                        Button(Localized.Settings.flockForget, action: forget)
                            .buttonStyle(.plain)
                            .foregroundStyle(.secondary)
                            .jayjayFont(11)
                            .accessibilityIdentifier(AID.Settings.flockForget)
                    } else if flock.isUnlocked {
                        FlockUnlockButton(title: Localized.Settings.flockVerify)
                            .controlSize(.small)
                        FlockCodeButton()
                            .controlSize(.small)
                    }
                }
                if flock.isUnlocked {
                    Text(flock.member == nil ? Localized.Settings.flockDebugDetail : Localized.Settings.flockThanks)
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                }
                if let forgetError {
                    Text(forgetError).jayjayFont(11).foregroundStyle(.red)
                }
            }
            Button(action: replay) { feather }
                .buttonStyle(.plain)
                .accessibilityLabel(displayName)
                .accessibilityHint(Localized.Settings.flockReplay)
                .accessibilityIdentifier(AID.Settings.flockFeather)
                .help(Localized.Settings.flockReplay)
            if flock.member != nil {
                Text(Localized.Settings.flockKeep).jayjayFont(11).foregroundStyle(.secondary)
            }
            VStack(alignment: .leading, spacing: 12) {
                Text(Localized.Settings.flockDockIcon).jayjayFont(13, weight: .semibold)
                FlockIconPicker()
                Text(Localized.Settings.flockDockIconDetail)
                    .jayjayFont(11)
                    .foregroundStyle(.secondary)
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 12))
        }
        .padding(24)
        .frame(maxWidth: .infinity)
    }

    private var displayName: String {
        if flock.member != nil {
            flock.name
        } else {
            flock.isUnlocked ? Localized.Settings.flockDebugBuild : Localized.Settings.flockYourFeather
        }
    }

    private var feather: some View {
        VStack(spacing: 8) {
            Text(Localized.Settings.flockTitle).jayjayFont(11, weight: .semibold)
            if let featherImage = Self.featherImage {
                Image(nsImage: featherImage)
                    .resizable()
                    .interpolation(.high)
                    .frame(width: 130, height: 130)
                    .accessibilityHidden(true)
            }
            Text(displayName).jayjayFont(19, weight: .medium, design: .serif)
                .lineLimit(2)
                .multilineTextAlignment(.center)
                .help(displayName)
            Text(Localized.Settings.flockFeatherTitle).jayjayFont(9, weight: .semibold)
        }
        .foregroundStyle(Color(red: 0.12, green: 0.22, blue: 0.38))
        .padding(20)
        .frame(width: 230)
        .background(Color(red: 0.98, green: 0.95, blue: 0.87), in: RoundedRectangle(cornerRadius: 12))
        .rotationEffect(.degrees(flying ? -5 : 0))
        .scaleEffect(flying ? 1.04 : 1)
        .shadow(color: .black.opacity(0.12), radius: 8, y: 4)
    }

    private func forget() {
        forgetError = nil
        Task {
            do {
                try await flock.forget()
            } catch {
                forgetError = error.friendlyDescription
            }
        }
    }

    private func replay() {
        guard flock.isUnlocked, !reduceMotion, !flying else { return }
        withAnimation(.spring(duration: 0.3)) { flying = true }
        Task {
            try? await Task.sleep(for: .milliseconds(300))
            withAnimation(.spring(duration: 0.4)) { flying = false }
        }
    }
}
