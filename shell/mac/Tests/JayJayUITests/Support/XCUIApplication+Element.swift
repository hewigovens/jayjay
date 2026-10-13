import XCTest

extension XCUIApplication {
    func element(_ identifier: String) -> XCUIElement {
        descendants(matching: .any)[identifier]
    }
}
