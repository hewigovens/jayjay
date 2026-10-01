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

    private static func post(_ type: CGEventType, at point: CGPoint, button: CGMouseButton, clickState: Int = 0) {
        guard let event = CGEvent(mouseEventSource: nil, mouseType: type, mouseCursorPosition: point, mouseButton: button) else { return }
        if clickState > 0 {
            event.setIntegerValueField(.mouseEventClickState, value: Int64(clickState))
        }
        event.flags = []
        event.post(tap: .cghidEventTap)
    }
}
