import AppKit
import SwiftUI

struct SelectionContextMenu<Content: View>: NSViewRepresentable {
    let openRequest: UUID?
    let onOpen: () -> Void
    let content: Content

    func makeNSView(context: Context) -> SelectionContextMenuView {
        let view = SelectionContextMenuView()
        context.coordinator.installClicks(on: view)
        return view
    }

    func updateNSView(_ view: SelectionContextMenuView, context: Context) {
        view.makeMenu = {
            let menu = NSHostingMenu(rootView: content)
            menu.update()
            return menu
        }
        context.coordinator.isEnabled = context.environment.isEnabled
        guard let openRequest, openRequest != context.coordinator.openedRequest, context.coordinator.isEnabled else { return }
        context.coordinator.openedRequest = openRequest
        DispatchQueue.main.async { [weak view] in
            onOpen()
            guard let view, view.window != nil, let menu = view.makeMenu?() else { return }
            view.scrollToVisible(view.bounds)
            menu.popUp(positioning: nil, at: NSPoint(x: view.bounds.midX, y: view.bounds.midY), in: view)
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator {
        var isEnabled = true
        var openedRequest: UUID?
        private var clickMonitor: Any?

        func installClicks(on view: SelectionContextMenuView) {
            clickMonitor = NSEvent.addLocalMonitorForEvents(matching: [.rightMouseDown, .leftMouseDown]) { [weak self, weak view] event in
                guard let self, isEnabled, let view, let window = view.window, event.window === window,
                      window.attachedSheet == nil, NSApp.modalWindow == nil,
                      event.type == .rightMouseDown || event.modifierFlags.contains(.control),
                      view.bounds.intersection(view.visibleRect).contains(view.convert(event.locationInWindow, from: nil)),
                      let scrollView = view.enclosingScrollView,
                      window.contentView?.superview?.hitTest(event.locationInWindow)?.isDescendant(of: scrollView) == true,
                      window.contentView?.hasContextMenuExclusion(at: event.locationInWindow) != true,
                      let menu = view.makeMenu?()
                else { return event }
                menu.popUp(positioning: nil, at: view.convert(event.locationInWindow, from: nil), in: view)
                return nil
            }
        }

        deinit {
            if let clickMonitor {
                NSEvent.removeMonitor(clickMonitor)
            }
        }
    }
}

final class SelectionContextMenuView: NSView {
    var makeMenu: (() -> NSMenu)?
}
