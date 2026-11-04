import JayJayCore
import JayJayDiffUI
import SwiftUI

struct ThemePreview: View {
    let scheme: ColorScheme
    @Environment(AppSettings.self) private var settings
    @Environment(\.jayjayTypography) private var typography
    @State private var diffHeight: CGFloat?

    private static let specimen = fontSpecimen()

    var body: some View {
        let specimen = Self.specimen
        let palettes = settings.diffPalettes
        let colors = settings.appThemes.colors(for: scheme)
        VStack(spacing: 0) {
            VStack(spacing: 0) {
                ForEach(Array(specimen.changes.enumerated()), id: \.offset) { index, change in
                    changeRow(
                        change,
                        hasTop: index > 0,
                        hasBottom: index + 1 < specimen.changes.count,
                        colors: colors
                    )
                }
            }
            .padding(.vertical, 4)
            .background(colors.navigationBackground)

            Divider()

            VStack(spacing: 0) {
                ForEach(specimen.files, id: \.path) { file in
                    fileRow(file, colors: colors)
                }
            }
            .padding(.vertical, 4)
            .background(colors.readingBackground)

            Divider()

            NativeDiffView(diff: specimen.diff, onContentHeightChanged: { diffHeight = $0 })
                .environment(\.diffTypography, typography.diffTypography)
                .environment(\.diffPalettes, palettes)
                .environment(\.colorScheme, scheme)
                .frame(height: diffHeight ?? specimen.diff.estimatedCardHeight(typography: typography))
                .padding(.vertical, 4)
                .background(colors.readingBackground)
        }
        .environment(\.colorScheme, scheme)
        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .stroke(Color.primary.opacity(0.12), lineWidth: 1)
        )
        .frame(maxWidth: .infinity, alignment: .topLeading)
        .accessibilityElement(children: .contain)
        .accessibilityIdentifier(AID.Settings.preview)
    }

    private func changeRow(_ change: SpecimenChange, hasTop: Bool, hasBottom: Bool, colors: AppColors) -> some View {
        HStack(spacing: 8) {
            graphLane(hasTop: hasTop, hasBottom: hasBottom, isWorkingCopy: change.isWorkingCopy, colors: colors)
            HStack(alignment: .firstTextBaseline, spacing: 5) {
                Text(change.changeId.highlighted(font: typography.identifierFont, prefixColor: colors.changeIdPrefix))
                if change.isWorkingCopy {
                    chip("@", tint: colors.workspace.opacity(scheme == .dark ? 0.15 : 0.18), foreground: colors.workspace)
                }
                if let bookmark = change.bookmark {
                    chip(
                        bookmark,
                        tint: colors.bookmark.opacity(scheme == .dark ? 0.16 : 0.14),
                        systemImage: "bookmark",
                        iconColor: colors.bookmark
                    )
                }
                Text(change.summary)
                    .jayjayFont(12, weight: change.isWorkingCopy ? .medium : .regular)
                    .lineLimit(1)
            }
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 10)
        .frame(minHeight: 24)
    }

    private func graphLane(hasTop: Bool, hasBottom: Bool, isWorkingCopy: Bool, colors: AppColors) -> some View {
        Canvas { ctx, size in
            let midX = size.width / 2
            let midY = size.height / 2
            if hasTop {
                var path = Path()
                path.move(to: CGPoint(x: midX, y: 0))
                path.addLine(to: CGPoint(x: midX, y: midY))
                ctx.stroke(path, with: .color(colors.graphLine), style: StrokeStyle(lineWidth: 1.5))
            }
            if hasBottom {
                var path = Path()
                path.move(to: CGPoint(x: midX, y: midY))
                path.addLine(to: CGPoint(x: midX, y: size.height))
                ctx.stroke(path, with: .color(colors.graphLine), style: StrokeStyle(lineWidth: 1.5))
            }
            let r: CGFloat = 3.5
            let rect = CGRect(x: midX - r, y: midY - r, width: r * 2, height: r * 2)
            ctx.fill(
                Path(ellipseIn: rect),
                with: .color(isWorkingCopy ? Color.accentColor : Color.secondary)
            )
        }
        .frame(width: 12, height: 24)
    }

    private func chip(
        _ title: String,
        tint: Color,
        foreground: Color = .primary,
        systemImage: String? = nil,
        iconColor: Color? = nil
    ) -> some View {
        HStack(alignment: .firstTextBaseline, spacing: 3) {
            if let systemImage {
                Image(systemName: systemImage)
                    .jayjayFont(10, weight: .semibold)
                    .imageScale(.small)
                    .foregroundStyle(iconColor ?? .secondary)
            }
            Text(title)
                .jayjayFont(10, weight: .semibold)
                .foregroundStyle(foreground)
                .lineLimit(1)
        }
        .padding(.horizontal, 5)
        .padding(.vertical, 2)
        .background(tint, in: Capsule())
    }

    private func fileRow(_ file: SpecimenFile, colors: AppColors) -> some View {
        let directory = (file.path as NSString).deletingLastPathComponent + "/"
        let fileName = (file.path as NSString).lastPathComponent
        let letter = switch file.hunkType {
            case .added: "A"
            case .removed: "D"
            case .modified: "M"
            case .renamed: "R"
            case .copied: "C"
        }
        return HStack(spacing: 8) {
            Text(letter)
                .jayjayFont(11, weight: .bold, design: .monospaced)
                .foregroundStyle(colors.fileColor(for: file.hunkType))
                .frame(width: 14, alignment: .leading)
            Text("\(Text(directory).foregroundStyle(.secondary))\(Text(fileName).fontWeight(.medium))")
                .jayjayFont(12)
                .lineLimit(1)
            Spacer(minLength: 0)
        }
        .padding(.horizontal, 10)
        .frame(minHeight: 22)
    }
}
