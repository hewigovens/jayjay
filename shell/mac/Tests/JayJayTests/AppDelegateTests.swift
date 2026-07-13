import AppKit
@testable import JayJay
import XCTest

@MainActor
final class AppDelegateTests: XCTestCase {
    func testDockOpenRepositoryPickerDispatchesWithoutSender() async throws {
        let appDelegate = AppDelegate()
        let opened = expectation(description: "repository picker opened")
        appDelegate.openRepositoryPicker = { opened.fulfill() }

        let menu = try XCTUnwrap(appDelegate.applicationDockMenu(NSApp))
        let item = try XCTUnwrap(menu.items.first)

        try sendDockAction(item)
        await fulfillment(of: [opened], timeout: 1)
    }

    func testDockRecentRepositoryDispatchesWithoutSender() async throws {
        let path = "/tmp/example-repo"
        let appDelegate = AppDelegate()
        let opened = expectation(description: "recent repository opened")
        appDelegate.recentReposProvider = { [path] }
        appDelegate.openHandler = { openedPath in
            XCTAssertEqual(openedPath, path)
            opened.fulfill()
        }

        let menu = try XCTUnwrap(appDelegate.applicationDockMenu(NSApp))
        let recentMenu = try XCTUnwrap(menu.items.first(where: { $0.title == "Recent Repositories" })?.submenu)
        let item = try XCTUnwrap(recentMenu.items.first)

        try sendDockAction(item)
        await fulfillment(of: [opened], timeout: 1)
    }

    private func sendDockAction(_ item: NSMenuItem) throws {
        let action = try XCTUnwrap(item.action)
        let target = try XCTUnwrap(item.target)
        XCTAssertTrue(NSApp.sendAction(action, to: target, from: nil))
    }
}
