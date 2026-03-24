import AppKit

enum DebugBadge {
    static func apply() {
        #if DEBUG
            let tile = NSApplication.shared.dockTile
            let iconView = NSImageView(frame: NSRect(x: 0, y: 0, width: tile.size.width, height: tile.size.height))
            iconView.image = NSApplication.shared.applicationIconImage
            let badge = NSTextField(labelWithString: "βeta")
            badge.font = NSFont.boldSystemFont(ofSize: 24)
            badge.textColor = .white
            badge.backgroundColor = .systemOrange
            badge.isBezeled = false
            badge.alignment = .center
            badge.sizeToFit()
            badge.frame = NSRect(
                x: tile.size.width - badge.frame.width - 4,
                y: 2,
                width: badge.frame.width + 8,
                height: badge.frame.height + 2
            )
            badge.wantsLayer = true
            badge.layer?.cornerRadius = badge.frame.height / 2
            badge.layer?.masksToBounds = true
            let container = NSView(frame: iconView.frame)
            container.addSubview(iconView)
            container.addSubview(badge)
            tile.contentView = container
            tile.display()
        #endif
    }
}
