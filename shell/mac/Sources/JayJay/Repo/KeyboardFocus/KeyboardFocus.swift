import AppKit
import Observation

/// Window-level keyboard focus: the pane that owns j/k, and the registered control Tab has landed on, if any.
@MainActor @Observable
final class KeyboardFocus {
    var activePane: ActivePane = .dag {
        didSet { control = nil }
    }

    private(set) var control: KeyboardFocusStop?
    var isSuspended = false {
        didSet {
            if isSuspended {
                control = nil
            }
        }
    }

    @ObservationIgnored private var registrations: [KeyboardFocusStop: (token: UUID, action: () -> Void)] = [:]

    func register(_ stop: KeyboardFocusStop, token: UUID, action: @escaping () -> Void) {
        registrations[stop] = (token, action)
    }

    func unregister(_ stop: KeyboardFocusStop, token: UUID) {
        guard registrations[stop]?.token == token else { return }
        registrations[stop] = nil
        if control == stop {
            control = nil
        }
    }

    func updateInputFocus(_ stop: KeyboardFocusStop, isFocused: Bool) {
        guard !isSuspended else { return }
        if isFocused {
            control = stop
        } else if control == stop {
            control = nil
        }
    }

    func handleKey(_ event: NSEvent) -> Bool {
        let modifiers = event.modifierFlags.intersection(.deviceIndependentFlagsMask)
        if event.keyCode == KeyCode.tab, modifiers.isSubset(of: .shift) {
            // Swallowed even while suspended: an unhandled Tab enters AppKit's key-view loop, which differs per machine.
            if !isSuspended {
                move(backward: modifiers.contains(.shift))
            }
            return true
        }
        guard !isSuspended else { return false }
        guard let control, !control.isTextInput else { return false }
        switch event.keyCode {
            case KeyCode.space, KeyCode.returnKey, KeyCode.keypadEnter:
                registrations[control]?.action()
            case KeyCode.escape:
                self.control = nil
            default:
                return false
        }
        return true
    }

    private func move(backward: Bool) {
        let stops = KeyboardFocusStop.allCases.filter { $0 == .dag || registrations[$0] != nil }
        let current = control ?? (activePane == .dag ? .dag : .fileList)
        let index = stops.firstIndex(of: current) ?? 0
        let next = stops[(index + (backward ? stops.count - 1 : 1)) % stops.count]
        NSApp.keyWindow?.makeFirstResponder(nil)
        switch next {
            case .dag:
                activePane = .dag
            case .fileList:
                registrations[.fileList]?.action()
            default:
                control = next
                if next.isTextInput {
                    registrations[next]?.action()
                }
        }
    }
}
