import AppKit
import SwiftUI

/// Tag the window with the repo path so reopening finds it while loading/erroring instead of duplicating.
struct WindowRepresentedURL: NSViewRepresentable {
    let path: String

    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        apply(to: view)
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        apply(to: nsView)
    }

    private func apply(to view: NSView) {
        let url = URL(fileURLWithPath: path)
        DispatchQueue.main.async {
            view.window?.representedURL = url
        }
    }
}
