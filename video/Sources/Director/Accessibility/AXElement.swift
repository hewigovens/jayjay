import ApplicationServices
import Foundation

struct AXElement: Equatable {
    let ref: AXUIElement

    static func == (lhs: AXElement, rhs: AXElement) -> Bool {
        CFEqual(lhs.ref, rhs.ref)
    }

    struct Summary {
        var role: String?
        var identifier: String?
        var title: String?
        var description: String?
        var children: [AXElement] = []
    }

    static func application(_ pid: pid_t) -> AXElement {
        // Set system-wide so it covers every element; children otherwise wait 6 s on a busy app.
        AXUIElementSetMessagingTimeout(AXUIElementCreateSystemWide(), 2)
        return AXElement(ref: AXUIElementCreateApplication(pid))
    }

    static var focusedApplicationPID: pid_t? {
        AXElement(ref: AXUIElementCreateSystemWide()).element(kAXFocusedApplicationAttribute)?.pid
    }

    static func element(at point: CGPoint) -> AXElement? {
        var element: AXUIElement?
        guard AXUIElementCopyElementAtPosition(AXUIElementCreateSystemWide(), Float(point.x), Float(point.y), &element) == .success, let element else { return nil }
        return AXElement(ref: element)
    }

    var pid: pid_t? {
        var pid: pid_t = 0
        return AXUIElementGetPid(ref, &pid) == .success ? pid : nil
    }

    var topLevel: AXElement {
        element(kAXTopLevelUIElementAttribute) ?? self
    }

    func isInside(_ ancestor: AXElement) -> Bool {
        var current: AXElement? = self
        for _ in 0 ..< 64 {
            guard let element = current else { return false }
            if element == ancestor {
                return true
            }
            current = element.element(kAXParentAttribute)
        }
        return false
    }

    private func element(_ attribute: String) -> AXElement? {
        guard let value = value(attribute), CFGetTypeID(value) == AXUIElementGetTypeID() else { return nil }
        return AXElement(ref: value as! AXUIElement)
    }

    func value(_ attribute: String) -> CFTypeRef? {
        var value: CFTypeRef?
        return AXUIElementCopyAttributeValue(ref, attribute as CFString, &value) == .success ? value : nil
    }

    func string(_ attribute: String) -> String? {
        value(attribute) as? String
    }

    func perform(_ action: String) {
        AXUIElementPerformAction(ref, action as CFString)
    }

    @discardableResult
    func set(_ attribute: String, to value: CFTypeRef) -> Bool {
        AXUIElementSetAttributeValue(ref, attribute as CFString, value) == .success
    }

    var windows: [AXElement] {
        (value(kAXWindowsAttribute) as? [AXUIElement] ?? []).map(AXElement.init)
    }

    var frame: CGRect? {
        guard let position = axValue(kAXPositionAttribute), let size = axValue(kAXSizeAttribute) else { return nil }
        var origin = CGPoint.zero
        var extent = CGSize.zero
        guard AXValueGetValue(position, .cgPoint, &origin), AXValueGetValue(size, .cgSize, &extent) else { return nil }
        return CGRect(origin: origin, size: extent)
    }

    func setFrame(_ frame: CGRect) {
        var origin = frame.origin
        var size = frame.size
        if let position = AXValueCreate(.cgPoint, &origin) {
            set(kAXPositionAttribute, to: position)
        }
        if let extent = AXValueCreate(.cgSize, &size) {
            set(kAXSizeAttribute, to: extent)
        }
    }

    func summary() -> Summary {
        var values: CFArray?
        let attributes = [kAXRoleAttribute, "AXIdentifier", kAXTitleAttribute, kAXDescriptionAttribute, kAXChildrenAttribute] as CFArray
        guard AXUIElementCopyMultipleAttributeValues(ref, attributes, AXCopyMultipleAttributeOptions(rawValue: 0), &values) == .success,
              let list = values as? [Any], list.count == 5
        else { return Summary() }
        return Summary(
            role: list[0] as? String,
            identifier: list[1] as? String,
            title: list[2] as? String,
            description: list[3] as? String,
            children: (list[4] as? [AXUIElement] ?? []).map(AXElement.init)
        )
    }

    /// Depth first, like XCUITest's firstMatch.
    func descendants() -> AnySequence<(element: AXElement, summary: Summary)> {
        AnySequence { () -> AnyIterator<(element: AXElement, summary: Summary)> in
            var stack = [self]
            return AnyIterator {
                while let element = stack.popLast() {
                    let summary = element.summary()
                    guard summary.role != kAXMenuBarRole else { continue }
                    stack.append(contentsOf: summary.children.reversed())
                    return (element, summary)
                }
                return nil
            }
        }
    }

    func bounds(ofLineMatching pattern: NSRegularExpression) -> CGRect? {
        guard let text = string(kAXValueAttribute) else { return nil }
        let lines = text.components(separatedBy: "\n")
        var location = 0
        for (index, line) in lines.enumerated() {
            let length = (line as NSString).length
            if pattern.firstMatch(in: line, range: NSRange(location: 0, length: length)) != nil {
                if let bounds = bounds(of: CFRange(location: location, length: length)), bounds.height > 0 {
                    return bounds
                }
                guard let frame else { return nil }
                let height = frame.height / CGFloat(lines.count)
                return CGRect(x: frame.minX, y: frame.minY + CGFloat(index) * height, width: frame.width, height: height)
            }
            location += length + 1
        }
        return nil
    }

    private func bounds(of range: CFRange) -> CGRect? {
        var range = range
        guard let parameter = AXValueCreate(.cfRange, &range) else { return nil }
        var result: CFTypeRef?
        guard AXUIElementCopyParameterizedAttributeValue(ref, kAXBoundsForRangeParameterizedAttribute as CFString, parameter, &result) == .success,
              let value = result, CFGetTypeID(value) == AXValueGetTypeID()
        else { return nil }
        var rect = CGRect.zero
        return AXValueGetValue(value as! AXValue, .cgRect, &rect) ? rect : nil
    }

    private func axValue(_ attribute: String) -> AXValue? {
        guard let value = value(attribute), CFGetTypeID(value) == AXValueGetTypeID() else { return nil }
        return (value as! AXValue)
    }
}
