import AppKit

public extension NSTextContainer {
    /// Every TextKit 1 view builds its storage here so symbols such as ↩ keep a text font.
    func makeTextKit1Storage(_ layoutManager: NSLayoutManager = NSLayoutManager()) -> NSTextStorage {
        layoutManager.addTextContainer(self)
        let storage = NSTextStorage()
        storage.delegate = TextPresentationFallback.shared
        storage.addLayoutManager(layoutManager)
        return storage
    }

    func makeTextKit1View<View: NSTextView>(
        _ layoutManager: NSLayoutManager = NSLayoutManager(),
        _ makeView: (NSTextContainer) -> View
    ) -> View {
        let storage = makeTextKit1Storage(layoutManager)
        // Nothing owns the storage until the text view adopts it.
        return withExtendedLifetime(storage) { makeView(self) }
    }
}
