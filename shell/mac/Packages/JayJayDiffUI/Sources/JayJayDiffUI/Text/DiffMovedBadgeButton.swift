import AppKit

final class DiffMovedBadgeButton: NSButton {
    private struct FontFit {
        let source: NSFont
        let height: CGFloat
        let fitted: NSFont
    }

    private static var fontFit: FontFit?
    static let trailingInset: CGFloat = 8
    private static let measurementCell: NSButtonCell = {
        let cell = NSButtonCell(textCell: "")
        cell.isBordered = false
        return cell
    }()

    static func reservedWidth(for title: String, font: NSFont) -> CGFloat {
        measurementCell.attributedTitle = attributedTitle(for: title, font: font)
        return measurementCell.cellSize.width + trailingInset + 8
    }

    private static func attributedTitle(for title: String, font: NSFont) -> NSAttributedString {
        NSAttributedString(string: " \(title) ", attributes: [
            .font: font,
            .foregroundColor: NSColor.controlAccentColor
        ])
    }

    static func fittedFont(_ font: NSFont, maxHeight: CGFloat) -> NSFont {
        if let cached = fontFit, cached.source == font, cached.height == maxHeight {
            return cached.fitted
        }
        // Arrow-and-line-number titles share font metrics, so fitting once serves every badge at this row height.
        var fitted = font
        while true {
            measurementCell.attributedTitle = attributedTitle(for: "→ L0", font: fitted)
            if measurementCell.cellSize.height <= maxHeight || fitted.pointSize <= 1 {
                break
            }
            fitted = NSFontManager.shared.convert(font, toSize: max(1, fitted.pointSize - 0.5))
        }
        fontFit = FontFit(source: font, height: maxHeight, fitted: fitted)
        return fitted
    }

    private struct Configuration {
        let title: String
        let font: NSFont
        let size: NSSize
    }

    private var configuration: Configuration?
    private var onPress: (() -> Void)?

    init() {
        super.init(frame: .zero)
        isBordered = false
        wantsLayer = true
        target = self
        action = #selector(press)
        setAccessibilityIdentifier("diff-moved-badge")
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var wantsUpdateLayer: Bool {
        true
    }

    override func updateLayer() {
        layer?.cornerRadius = 4
        layer?.backgroundColor = NSColor.controlAccentColor.withAlphaComponent(0.14).cgColor
    }

    override func resetCursorRects() {
        addCursorRect(bounds, cursor: .pointingHand)
    }

    @discardableResult
    func configure(title: String, font: NSFont, onPress: @escaping () -> Void) -> NSSize {
        self.onPress = onPress
        if let cached = configuration, cached.title == title, cached.font == font {
            return cached.size
        }
        attributedTitle = Self.attributedTitle(for: title, font: font)
        setAccessibilityLabel("Jump to moved copy at \(title.dropFirst(2))")
        let size = fittingSize
        configuration = Configuration(title: title, font: font, size: size)
        return size
    }

    @objc private func press() {
        onPress?()
    }
}
