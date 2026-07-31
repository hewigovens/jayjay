import AppKit

final class DockIconView: NSImageView {
    private weak var dockTile: NSDockTile?
    private let lightImage: NSImage
    private let darkImage: NSImage
    private var hostAppearanceObservation: NSKeyValueObservation?
    #if DEBUG
        private let developmentBadge = DevelopmentBadgeView()
    #endif

    static func install(on dockTile: NSDockTile, bundle: Bundle) {
        guard let iconView = DockIconView(
            frame: NSRect(origin: .zero, size: dockTile.size),
            dockTile: dockTile,
            bundle: bundle
        ) else { return }
        dockTile.badgeLabel = nil
        dockTile.contentView = iconView
        dockTile.display()
    }

    init?(frame frameRect: NSRect, dockTile: NSDockTile, bundle: Bundle) {
        guard let lightURL = bundle.url(forResource: "jayjay-dock-light", withExtension: "png"),
              let darkURL = bundle.url(forResource: "jayjay-dock-dark", withExtension: "png"),
              let lightImage = NSImage(contentsOf: lightURL),
              let darkImage = NSImage(contentsOf: darkURL)
        else { return nil }

        self.dockTile = dockTile
        self.lightImage = lightImage
        self.darkImage = darkImage
        super.init(frame: frameRect)
        imageAlignment = .alignCenter
        imageScaling = .scaleAxesIndependently
        autoresizingMask = [.width, .height]
        #if DEBUG
            addSubview(developmentBadge)
        #endif
        let hostApplication = NSApplication.shared
        // A Dock tile content view is detached from the host's view hierarchy, so apply each observed appearance explicitly.
        applyHostAppearance(hostApplication.effectiveAppearance)
        hostAppearanceObservation = hostApplication.observe(\.effectiveAppearance, options: [.new]) { [weak self] application, _ in
            DispatchQueue.main.async { [weak self] in
                self?.applyHostAppearance(application.effectiveAppearance)
            }
        }
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func layout() {
        super.layout()
        #if DEBUG
            let side = min(bounds.width, bounds.height)
            let iconSide = side * 0.805
            let iconRect = NSRect(
                x: bounds.midX - iconSide / 2,
                y: bounds.midY - iconSide / 2,
                width: iconSide,
                height: iconSide
            )
            developmentBadge.frame = NSRect(
                x: iconRect.maxX - side * 0.40,
                y: iconRect.minY + side * 0.015,
                width: side * 0.38,
                height: side * 0.25
            )
        #endif
    }

    private func applyHostAppearance(_ appearance: NSAppearance) {
        self.appearance = appearance
        image = appearance.bestMatch(from: [.aqua, .darkAqua]) == .darkAqua ? darkImage : lightImage
        dockTile?.display()
    }
}

#if DEBUG
    private final class DevelopmentBadgeView: NSView {
        override func draw(_ dirtyRect: NSRect) {
            super.draw(dirtyRect)

            NSColor.systemOrange.setFill()
            NSBezierPath(roundedRect: bounds, xRadius: bounds.height / 2, yRadius: bounds.height / 2).fill()

            let text = NSAttributedString(
                string: "DEV",
                attributes: [
                    .font: NSFont.boldSystemFont(ofSize: bounds.height * 0.58),
                    .foregroundColor: NSColor.white
                ]
            )
            let textSize = text.size()
            text.draw(at: NSPoint(
                x: (bounds.width - textSize.width) / 2,
                y: (bounds.height - textSize.height) / 2
            ))
        }
    }
#endif
