import CoreGraphics
import Foundation

/// Posted through the HID tap so hover and menu tracking behave as for a real mouse.
enum Pointer {
    static func click(at point: CGPoint, button: CGMouseButton, count: Int, check: () throws -> Void) async throws {
        post(.mouseMoved, at: point, button: .left)
        try? await Task.sleep(for: .milliseconds(120))
        let (down, up): (CGEventType, CGEventType) = button == .right ? (.rightMouseDown, .rightMouseUp) : (.leftMouseDown, .leftMouseUp)
        for click in 1 ... count {
            try check()
            post(down, at: point, button: button, clickState: click)
            try? await Task.sleep(for: .milliseconds(40))
            post(up, at: point, button: button, clickState: click)
            try? await Task.sleep(for: .milliseconds(80))
        }
    }

    /// Many one-line wheel events, so the take shows a smooth scroll rather than a jump.
    static func scroll(at point: CGPoint, lines: Int, check: () throws -> Void) async throws {
        post(.mouseMoved, at: point, button: .left)
        try? await Task.sleep(for: .milliseconds(120))
        let step: Int32 = lines >= 0 ? -1 : 1
        for _ in 0 ..< abs(lines) {
            try check()
            guard let event = CGEvent(scrollWheelEvent2Source: nil, units: .line, wheelCount: 1, wheel1: step, wheel2: 0, wheel3: 0) else { return }
            event.location = point
            event.post(tap: .cghidEventTap)
            try? await Task.sleep(for: .milliseconds(30))
        }
    }

    private static func post(_ type: CGEventType, at point: CGPoint, button: CGMouseButton, clickState: Int = 0) {
        guard let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: button) else { return }
        if clickState > 0 {
            event.setIntegerValueField(.mouseEventClickState, value: Int64(clickState))
        }
        event.flags = []
        event.post(tap: .cghidEventTap)
    }
}
