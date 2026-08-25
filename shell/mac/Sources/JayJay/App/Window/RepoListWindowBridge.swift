import SwiftUI

/// openWindow and dismissWindow only exist in a view's environment; the manager is driven from AppKit callbacks and menus.
struct RepoListWindowBridge: View {
    let windowManager: RepoWindowManager
    let scene: String
    var onRegistered: () -> Void = {}

    @Environment(\.openWindow) private var openWindow
    @Environment(\.dismissWindow) private var dismissWindow

    var body: some View {
        Color.clear
            .frame(width: 0, height: 0)
            .onAppear {
                windowManager.setWindowActions(
                    presenting: scene,
                    openWindow: { id, value in
                        if let value {
                            openWindow(id: id, value: value)
                        } else {
                            openWindow(id: id)
                        }
                    },
                    dismissWindow: { dismissWindow(id: $0) }
                )
                onRegistered()
            }
    }
}
