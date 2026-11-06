import AppKit
import JayJayCore
import Observation

@Observable
final class ShortcutRecorder {
    private(set) var commandID: String?
    private(set) var heldCaps: [String] = []
    private(set) var pending: ShortcutEdit?
    private(set) var pendingChord: ShortcutChord?
    private(set) var message: String?
    private(set) var canReplace = false
    private(set) var isRecording = false
    private let store: ShortcutStore
    @ObservationIgnored private weak var focusView: NSView?
    @ObservationIgnored private var monitor: Any?

    init(store: ShortcutStore) {
        self.store = store
    }

    func begin(id: String, view: NSView) {
        cancel()
        commandID = id
        focusView = view
        view.window?.makeFirstResponder(view)
        isRecording = true
        monitor = NSEvent.addLocalMonitorForEvents(matching: [.keyDown, .flagsChanged, .leftMouseDown, .rightMouseDown]) { [weak self] event in
            guard let self else { return event }
            return capture(event)
        }
    }

    func capture(_ event: NSEvent) -> NSEvent? {
        guard isRecording || pending != nil else { return event }
        guard let view = focusView, let window = view.window else {
            cancel()
            return event
        }
        if window.isKeyWindow, event.type == .keyDown, event.keyCode == KeyCode.escape {
            cancel()
            return nil
        }
        guard pending == nil else { return event }
        if event.type == .leftMouseDown || event.type == .rightMouseDown {
            if event.window !== window || !view.bounds.contains(view.convert(event.locationInWindow, from: nil)) {
                cancel()
            }
            return event
        }
        guard window.isKeyWindow, window.firstResponder === view else {
            cancel()
            return event
        }
        if event.type == .flagsChanged {
            heldCaps = shortcutModifierCaps(
                ctrl: event.modifierFlags.contains(.control),
                alt: event.modifierFlags.contains(.option),
                shift: event.modifierFlags.contains(.shift),
                cmd: event.modifierFlags.contains(.command)
            )
            return nil
        }
        guard let commandID else { return event }
        if [KeyCode.delete, KeyCode.forwardDelete].contains(event.keyCode), event.modifierFlags.isDisjoint(with: [.command, .control, .option, .shift]) {
            apply(.unbind, id: commandID)
        } else {
            apply(.assign(chord: event.shortcutText), id: commandID)
        }
        return nil
    }

    func apply(_ edit: ShortcutEdit, id: String, replace: Bool = false) {
        do {
            try store.apply(edit, to: id, replace: replace)
            cancel()
        } catch let error as ShortcutEditError {
            commandID = id
            switch error {
                case let .Invalid(kind):
                    message = kind.localizedDescription
                case let .Conflict(chord, commands), let .LockedConflict(chord, commands):
                    isRecording = false
                    pending = edit
                    pendingChord = chord
                    canReplace = if case .Conflict = error {
                        true
                    } else {
                        false
                    }
                    let names = commands.map { "\($0.title) (\($0.contextTitle))" }.joined(separator: ", ")
                    message = canReplace ? Localized.Shortcuts.conflict(actions: names) : Localized.Shortcuts.lockedConflict(actions: names)
                case .LockedCommand, .UnknownCommand:
                    cancel()
            }
        } catch {
            message = error.localizedDescription
        }
    }

    func replace() {
        guard let pending, let commandID else { return }
        apply(pending, id: commandID, replace: true)
    }

    func cancel() {
        isRecording = false
        if let monitor {
            NSEvent.removeMonitor(monitor)
        }
        monitor = nil
        commandID = nil
        pending = nil
        pendingChord = nil
        heldCaps = []
        message = nil
        canReplace = false
    }

    deinit {
        if let monitor {
            NSEvent.removeMonitor(monitor)
        }
    }
}
