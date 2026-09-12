import SwiftUI

extension View {
    /// Registers the view as a Tab stop while it is visible and available; Space or Return on it runs `action`.
    func keyboardFocusStop(
        _ stop: KeyboardFocusStop,
        isAvailable: Bool = true,
        action: @escaping () -> Void
    ) -> some View {
        modifier(KeyboardFocusStopModifier(stop: stop, isAvailable: isAvailable, action: action))
    }
}

private struct KeyboardFocusStopModifier: ViewModifier {
    let stop: KeyboardFocusStop
    let isAvailable: Bool
    let action: () -> Void
    @Environment(KeyboardFocus.self) private var focus: KeyboardFocus?

    func body(content: Content) -> some View {
        content
            .overlay {
                if focus?.control == stop, !stop.isTextInput {
                    RoundedRectangle(cornerRadius: 4, style: .continuous)
                        .stroke(Color.accentColor, lineWidth: 1.5)
                        .padding(-3)
                }
            }
            .background {
                Registration(focus: focus, stop: stop, isAvailable: isAvailable, action: action)
                    .frame(width: 0, height: 0)
                    .allowsHitTesting(false)
            }
    }
}

/// Re-registers on every update so the stored action never captures stale view state, and unregisters exactly once on dismantle.
private struct Registration: NSViewRepresentable {
    let focus: KeyboardFocus?
    let stop: KeyboardFocusStop
    let isAvailable: Bool
    let action: () -> Void

    func makeNSView(context: Context) -> NSView {
        NSView()
    }

    func updateNSView(_: NSView, context: Context) {
        let coordinator = context.coordinator
        if coordinator.focus !== focus || coordinator.stop != stop, let previousStop = coordinator.stop {
            coordinator.focus?.unregister(previousStop, token: coordinator.token)
        }
        coordinator.focus = focus
        coordinator.stop = stop
        if isAvailable {
            focus?.register(stop, token: coordinator.token, action: action)
        } else {
            focus?.unregister(stop, token: coordinator.token)
        }
    }

    static func dismantleNSView(_: NSView, coordinator: Coordinator) {
        if let stop = coordinator.stop {
            coordinator.focus?.unregister(stop, token: coordinator.token)
        }
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator {
        let token = UUID()
        weak var focus: KeyboardFocus?
        var stop: KeyboardFocusStop?
    }
}
