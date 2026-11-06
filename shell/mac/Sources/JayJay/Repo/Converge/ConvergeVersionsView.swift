import JayJayCore
import SwiftUI

struct ConvergeVersionsView: View {
    let review: ConvergeReview
    let loadedCommitIds: Set<String>
    @Environment(\.jayjayTypography) private var typography

    var body: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(alignment: .firstTextBaseline, spacing: 10) {
                Text(Localized.Converge.versions).jayjayFont(13, weight: .semibold)
                Text(Localized.Converge.versionsHint).jayjayFont(11).foregroundStyle(.secondary)
            }
            if review.versions.contains(where: { !loadedCommitIds.contains($0.commitId) }) {
                Text(Localized.Converge.outsideFilter).jayjayFont(11).foregroundStyle(.secondary)
            }
            let layout = review.versions.count == 2 && typography.interfaceSize <= 14
                ? AnyLayout(HStackLayout(alignment: .top, spacing: 10))
                : AnyLayout(VStackLayout(alignment: .leading, spacing: 10))
            layout {
                ForEach(review.versions, id: \.commitId) { version in
                    card(version)
                }
            }
        }
    }

    private func card(_ version: ConvergeVersion) -> some View {
        let letter = review.letter(of: version.commitId)
        return VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 8) {
                Text(letter)
                    .jayjayFont(10, weight: .bold)
                    .foregroundStyle(.white)
                    .frame(width: 16, height: 16)
                    .background(Color.accentColor, in: RoundedRectangle(cornerRadius: 4))
                Text(String(version.commitId.prefix(12)))
                    .jayjayFont(12, weight: .semibold, design: .monospaced)
                    .foregroundStyle(Color.accentColor)
                    .textSelection(.enabled)
                Spacer()
                Button(Localized.Converge.keep(version: letter)) { review.keep(version.commitId) }
                    .buttonStyle(.plain)
                    .jayjayFont(11, weight: .medium)
                    .foregroundStyle(Color.accentColor)
                    .accessibilityIdentifier(AID.Converge.keep(letter))
            }
            Text(version.subject.isEmpty ? Localized.Converge.noDescription : version.subject)
                .jayjayFont(12, weight: .semibold)
                .fixedSize(horizontal: false, vertical: true)
            Text(details(version))
                .jayjayFont(11)
                .foregroundStyle(.secondary)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(nsColor: .textBackgroundColor), in: RoundedRectangle(cornerRadius: 7))
        .overlay(RoundedRectangle(cornerRadius: 7).stroke(Color.primary.opacity(0.12)))
    }

    private func details(_ version: ConvergeVersion) -> String {
        ([version.author.name] + version.bookmarks + version.workspaces.map { "\($0)@" }
            + (version.isImmutable ? [Localized.Converge.immutable] : []))
            .joined(separator: " · ")
    }
}
