import XCTest

final class WorkspaceSidebarListScene: SceneBase {
    override class var fixtureName: String { "workspaces" }

    func testSidebarListsDefaultAndNamedWorkspaces() throws {
        let app = try XCTUnwrap(app)
        let sidebar = app.descendants(matching: .any)[AID.Workspace.sidebar]
        XCTAssertTrue(sidebar.waitForExistence(timeout: 10), "Workspace sidebar did not appear")
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.row("default")].waitForExistence(timeout: 5))
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.row("agent-pr")].exists)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.row("indexer")].exists)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.currentIndicator("default")].exists)
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.newWorkspace].exists)
        assertIdentityBarIsCompact(in: app, named: "default")
    }
}

final class WorkspaceSidebarRebindScene: SceneBase {
    override class var fixtureName: String { "workspaces-rebind" }

    func testSelectingNonCurrentWorkspaceRebindsThisWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1)
        let row = app.descendants(matching: .any)[AID.Workspace.row("agent-pr")]
        clickCenter(row, message: "agent-pr row missing")

        let rebound = app.windows["workspaces-rebind-agent-pr"]
        XCTAssertTrue(rebound.waitForExistence(timeout: 10), "Window did not rebind to agent-pr")
        XCTAssertEqual(app.windows.count, 1, "Rebind must not open a second window")
        XCTAssertTrue(
            app.descendants(matching: .any)[AID.Workspace.currentIndicator("agent-pr")]
                .waitForExistence(timeout: 5)
        )
        let identity = app.descendants(matching: .any)[AID.Workspace.identity]
        XCTAssertTrue(identity.waitForExistence(timeout: 5), "Workspace identity bar missing")
        XCTAssertTrue(
            identity.label.contains("agent-pr"),
            "Identity bar should name the selected workspace immediately"
        )
        assertIdentityBarIsCompact(in: app, named: "agent-pr")
    }
}

final class WorkspaceSidebarShowChangesScene: SceneBase {
    override class var fixtureName: String { "workspaces-show-changes" }

    func testShowChangesDoesNotChangeWorkingCopy() throws {
        let app = try XCTUnwrap(app)
        let row = app.descendants(matching: .any)[AID.Workspace.row("agent-pr")]
        rightClickCenter(row, message: "agent-pr row missing")
        let show = app.menuItems["Show Changes"]
        XCTAssertTrue(show.waitForExistence(timeout: 5), "Show Changes missing")
        show.click()

        let banner = app.descendants(matching: .any)[AID.Compare.banner]
        XCTAssertTrue(banner.waitForExistence(timeout: 8), "Compare banner did not appear")
        XCTAssertTrue(app.windows["workspaces-show-changes"].exists, "Window must stay on the bound workspace")
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.currentIndicator("default")].exists)
    }
}

final class WorkspaceSidebarForgetScene: SceneBase {
    override class var fixtureName: String { "workspaces-forget" }

    func testForgetConfirmsAndRemovesRow() throws {
        let app = try XCTUnwrap(app)
        let row = app.descendants(matching: .any)[AID.Workspace.row("indexer")]
        rightClickCenter(row, message: "indexer row missing")
        let forget = app.menuItems["Forget Workspace"]
        XCTAssertTrue(forget.waitForExistence(timeout: 5))
        forget.click()

        let confirm = app.buttons["Forget"].firstMatch
        XCTAssertTrue(confirm.waitForExistence(timeout: 5), "Forget confirmation missing")
        confirm.click()

        let gone = app.descendants(matching: .any)[AID.Workspace.row("indexer")]
        XCTAssertTrue(gone.waitForNonExistence(timeout: 8), "Forgotten workspace row stayed visible")
    }
}

final class WorkspaceSidebarToggleScene: SceneBase {
    override class var fixtureName: String { "workspaces" }

    func testShortcutTogglesSidebarVisibility() throws {
        let app = try XCTUnwrap(app)
        let sidebar = app.descendants(matching: .any)[AID.Workspace.sidebar]
        XCTAssertTrue(sidebar.waitForExistence(timeout: 10))

        keyStroke("w", modifiers: [.command, .option])
        XCTAssertTrue(sidebar.waitForNonExistence(timeout: 5), "⌥⌘W did not hide the sidebar")
        XCTAssertTrue(app.descendants(matching: .any)[AID.Workspace.rail].waitForExistence(timeout: 3))

        keyStroke("w", modifiers: [.command, .option])
        XCTAssertTrue(sidebar.waitForExistence(timeout: 5), "⌥⌘W did not show the sidebar")
    }
}

final class WorkspaceSidebarCreateScene: SceneBase {
    override class var fixtureName: String { "workspaces-create" }

    func testCreateSelectsInThisWindow() throws {
        let app = try XCTUnwrap(app)
        XCTAssertEqual(app.windows.count, 1)
        let newButton = app.descendants(matching: .any)[AID.Workspace.newWorkspace]
        clickCenter(newButton, message: "New Workspace missing")

        let field = app.textFields["Workspace name"]
        XCTAssertTrue(field.waitForExistence(timeout: 5))
        field.click()
        paste("fresh-ws")
        app.buttons["Create"].click()

        XCTAssertTrue(
            app.descendants(matching: .any)[AID.Workspace.row("fresh-ws")].waitForExistence(timeout: 10)
        )
        XCTAssertEqual(app.windows.count, 1, "Create must rebind this window, not open another")
    }
}

extension SceneBase {
    /// Catches the layout bug where the identity stripe ate the graph column.
    func assertIdentityBarIsCompact(in app: XCUIApplication, named name: String) {
        let identity = app.descendants(matching: .any)[AID.Workspace.identity]
        XCTAssertTrue(identity.waitForExistence(timeout: 5), "Workspace identity bar missing")
        let height = identity.frame.height
        let windowHeight = app.windows.firstMatch.frame.height
        XCTAssertGreaterThan(height, 8, "Identity bar should be visible")
        XCTAssertLessThan(height, 64, "Identity bar filled the column (\(height)pt)")
        XCTAssertLessThan(
            height,
            windowHeight * 0.15,
            "Identity bar took \(height)pt of a \(windowHeight)pt window"
        )
        XCTAssertTrue(identity.label.contains(name), "Identity bar should name \(name)")
    }
}
