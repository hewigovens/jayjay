import AppKit
import Observation

/// Window-level keyboard focus: the pane that owns j/k, and the registered control Tab has landed on, if any.
@MainActor @Observable
final class KeyboardFocus {
    var activePane: ActivePane = .dag {
        didSet {
            if isSidebarHidden, activePane == .dag {
                activePane = .fileColumn
            }
            control = nil
        }
    }

    /// A hidden sidebar stays mounted, so its stops are skipped here instead of unregistering.
    var isSidebarHidden = false {
        didSet {
            guard isSidebarHidden else { return }
            if activePane == .dag {
                activePane = .fileColumn
            } else if control?.isInSidebar == true {
                control = nil
            }
        }
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

    func focusPane(_ pane: ActivePane) {
        NSApp.keyWindow?.makeFirstResponder(nil)
        switch pane {
            case .dag:
                activePane = .dag
            case .fileColumn:
                registrations[.fileList]?.action()
        }
    }

    /// A field already on screen needs its registered action; one about to appear takes focus in `onAppear`.
    func focusInput(_ stop: KeyboardFocusStop) {
        updateInputFocus(stop, isFocused: true)
        registrations[stop]?.action()
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
        guard !isSuspended, let control, !control.ownsKeys else { return false }
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
        let stops = KeyboardFocusStop.allCases.filter {
            ($0 == .dag || registrations[$0] != nil) && !(isSidebarHidden && $0.isInSidebar)
        }
        guard let first = stops.first, let last = stops.last else { return }
        let current = control ?? (activePane == .dag ? .dag : .fileList)
        // A current stop that is not on screen still has its place in the cycle, so moving from it never skips a neighbour.
        let rank = { (stop: KeyboardFocusStop) in KeyboardFocusStop.allCases.firstIndex(of: stop) ?? 0 }
        let next = backward
            ? stops.last { rank($0) < rank(current) } ?? last
            : stops.first { rank($0) > rank(current) } ?? first
        NSApp.keyWindow?.makeFirstResponder(nil)
        switch next {
            case .dag:
                focusPane(.dag)
            case .fileList:
                focusPane(.fileColumn)
            default:
                control = next
                if next.ownsKeys {
                    registrations[next]?.action()
                }
        }
    }
}
