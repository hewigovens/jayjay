import AppKit
import SwiftUI

/// Applies a one-off configuration to the hosting window (tagging identity, representedURL, ...) once it exists.
struct WindowConfigurator: NSViewRepresentable {
    final class Coordinator {
        var configured = false
    }

    let configure: (NSWindow) -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> NSView {
        let view = NSView(frame: .zero)
        apply(to: view, once: context.coordinator)
        return view
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        apply(to: nsView, once: context.coordinator)
    }

    private func apply(to view: NSView, once coordinator: Coordinator) {
        DispatchQueue.main.async {
            guard !coordinator.configured, let window = view.window else { return }
            coordinator.configured = true
            configure(window)
        }
    }
}
