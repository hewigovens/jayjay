import AppKit
import JayJayCore
import SwiftUI

public extension DiffPreview {
    var imagePath: String? {
        if case let .image(path) = self {
            return path
        }
        return nil
    }
}

public struct ImageDiffView: View {
    public let oldPath: String?
    public let newPath: String?
    public let hunkType: HunkType

    @State private var oldImage: NSImage?
    @State private var newImage: NSImage?

    public init(oldPath: String?, newPath: String?, hunkType: HunkType) {
        self.oldPath = oldPath
        self.newPath = newPath
        self.hunkType = hunkType
    }

    public var body: some View {
        content
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .task(id: "\(oldPath ?? "")|\(newPath ?? "")") { await loadImages() }
    }

    @ViewBuilder
    private var content: some View {
        switch hunkType {
            case .added:
                imagePane(
                    image: newImage,
                    path: newPath,
                    label: "Added",
                    tint: .green,
                    showsLabel: false
                )
                .padding(16)
            case .removed:
                imagePane(
                    image: oldImage,
                    path: oldPath,
                    label: "Removed",
                    tint: .red,
                    showsLabel: false
                )
                .padding(16)
            case .renamed:
                // Usually content-identical — single pane avoids an empty "Before".
                imagePane(
                    image: newImage,
                    path: newPath,
                    label: "Renamed",
                    tint: .blue,
                    showsLabel: false
                )
                .padding(16)
            case .modified:
                HStack(spacing: 12) {
                    imagePane(image: oldImage, path: oldPath, label: "Before", tint: .red)
                    imagePane(image: newImage, path: newPath, label: "After", tint: .green)
                }
                .padding(16)
        }
    }

    private func imagePane(
        image: NSImage?,
        path: String?,
        label: String,
        tint: Color,
        showsLabel: Bool = true
    ) -> some View {
        VStack(spacing: 8) {
            if showsLabel {
                Text(label)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundStyle(tint)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 3)
                    .background(tint.opacity(0.12), in: Capsule())
            }

            ZStack {
                CheckerboardBackground()
                if let image {
                    // Clamp to natural size so small icons render 1:1 instead of upscaling.
                    Image(nsImage: image)
                        .resizable()
                        .interpolation(.high)
                        .scaledToFit()
                        .frame(maxWidth: image.size.width, maxHeight: image.size.height)
                        .padding(8)
                } else if path == nil {
                    Text("—")
                        .foregroundStyle(.secondary)
                } else {
                    ProgressView()
                        .controlSize(.small)
                }
            }
            .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
            .overlay(
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .stroke(Color.primary.opacity(0.12), lineWidth: 1)
            )

            metadata(for: image, path: path)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    @ViewBuilder
    private func metadata(for image: NSImage?, path: String?) -> some View {
        if let image, let path {
            let pixelSize = ImageDiffView.pixelSize(of: image)
            let fileSize = ImageDiffView.fileSize(at: path)
            Text("\(pixelSize) · \(fileSize)")
                .font(.system(size: 10, design: .monospaced))
                .foregroundStyle(.secondary)
                .lineLimit(1)
        } else {
            Text(" ")
                .font(.system(size: 10))
        }
    }

    private func loadImages() async {
        async let old = loadImage(at: oldPath)
        async let new = loadImage(at: newPath)
        let (loadedOld, loadedNew) = await (old, new)
        await MainActor.run {
            oldImage = loadedOld
            newImage = loadedNew
        }
    }

    nonisolated private func loadImage(at path: String?) async -> NSImage? {
        guard let path else { return nil }
        return await Task.detached { NSImage(contentsOfFile: path) }.value
    }

    private static func pixelSize(of image: NSImage) -> String {
        if let rep = image.representations.first {
            return "\(rep.pixelsWide)×\(rep.pixelsHigh) px"
        }
        return "\(Int(image.size.width))×\(Int(image.size.height)) pt"
    }

    private static func fileSize(at path: String) -> String {
        let bytes = (try? FileManager.default.attributesOfItem(atPath: path)[.size] as? Int) ?? 0
        return ByteCountFormatter.string(fromByteCount: Int64(bytes), countStyle: .file)
    }
}

/// 10pt checkerboard so transparent PNGs are obvious.
private struct CheckerboardBackground: View {
    var body: some View {
        Canvas { context, size in
            let tile: CGFloat = 10
            let dark = Color.primary.opacity(0.08)
            let light = Color.primary.opacity(0.02)
            context.fill(Path(CGRect(origin: .zero, size: size)), with: .color(light))
            var y: CGFloat = 0
            var row = 0
            while y < size.height {
                var x: CGFloat = (row % 2 == 0) ? 0 : tile
                while x < size.width {
                    let rect = CGRect(x: x, y: y, width: tile, height: tile)
                    context.fill(Path(rect), with: .color(dark))
                    x += tile * 2
                }
                y += tile
                row += 1
            }
        }
    }
}
