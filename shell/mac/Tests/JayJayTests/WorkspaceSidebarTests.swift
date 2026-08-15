@testable import JayJay
import AppKit
import JayJayCore
import SwiftUI
import XCTest

final class WorkspaceSidebarPolicyTests: XCTestCase {
    func testForgetIsDisabledForTheBoundWorkspace() {
        let current = WorkspaceInfo(
            name: "default",
            path: "/repo",
            isCurrent: true,
            wcCommitId: "aaa",
            parentCommitId: "bbb",
            timestampMillis: 1,
            changedFileCount: 0,
            description: "initial",
            pathExists: true
        )
        let other = WorkspaceInfo(
            name: "agent-pr",
            path: "/repo-agent",
            isCurrent: false,
            wcCommitId: "ccc",
            parentCommitId: "ddd",
            timestampMillis: 2,
            changedFileCount: 1,
            description: "work",
            pathExists: true
        )
        XCTAssertFalse(WorkspaceSidebarPolicy.canForget(current))
        XCTAssertTrue(WorkspaceSidebarPolicy.canForget(other))
        XCTAssertEqual(WorkspaceSidebarPolicy.workingCopySummary(other), "work")
        XCTAssertEqual(WorkspaceSidebarPolicy.fileCountVersusParent(other), "1 file vs parent")
        XCTAssertEqual(WorkspaceSidebarPolicy.fileCountVersusParent(current), "0 files vs parent")
        XCTAssertTrue(WorkspaceSidebarPolicy.canCompare(other, against: current))
        XCTAssertFalse(WorkspaceSidebarPolicy.canCompare(current, against: current))

        let flipped = WorkspaceSidebarPolicy.markingCurrent([current, other], name: "agent-pr")
        XCTAssertEqual(flipped.map(\.name), ["default", "agent-pr"])
        XCTAssertFalse(flipped[0].isCurrent)
        XCTAssertTrue(flipped[1].isCurrent)

        XCTAssertEqual(
            WorkspaceSidebarPolicy.identitySubtitle(other),
            "work · 1 file vs parent"
        )
        XCTAssertTrue(WorkspaceSidebarPolicy.isSamePath("/repo/../repo-agent", "/repo-agent"))
        XCTAssertEqual(
            WorkspaceSidebarPolicy.boundWorkspace(in: [current, other], repoPath: "/repo-agent")?.name,
            "agent-pr"
        )
        XCTAssertEqual(
            WorkspaceSidebarPolicy.boundWorkspace(in: [current, other], repoPath: "/repo")?.name,
            "default"
        )
        let stale = WorkspaceSidebarPolicy.mergingAdopted([current, other], current: flipped)
        XCTAssertEqual(stale.first(where: \.isCurrent)?.name, "agent-pr")

        XCTAssertTrue(WorkspaceSidebarPolicy.shouldDeferRebind(pulling: true, pushing: false))
        XCTAssertTrue(WorkspaceSidebarPolicy.shouldDeferRebind(pulling: false, pushing: true))
        XCTAssertFalse(WorkspaceSidebarPolicy.shouldDeferRebind(pulling: false, pushing: false))
        XCTAssertEqual(
            WorkspaceSidebarPolicy.identityStatus(pulling: true, pushing: false, switchPending: true),
            "Waiting for pull…"
        )
        XCTAssertEqual(
            WorkspaceSidebarPolicy.identityStatus(pulling: false, pushing: true, switchPending: true),
            "Waiting for push…"
        )
        XCTAssertNil(
            WorkspaceSidebarPolicy.identityStatus(pulling: true, pushing: false, switchPending: false)
        )
    }

    @MainActor
    func testIdentityBarHugsOneLineWhenTheColumnIsTall() {
        let workspace = WorkspaceInfo(
            name: "default",
            path: "/repo",
            isCurrent: true,
            wcCommitId: "aaa",
            parentCommitId: "bbb",
            timestampMillis: 1,
            changedFileCount: 0,
            description: "chore: prepare next task",
            pathExists: true
        )
        let measured = IdentityBarHeightBox()
        let laidOut = expectation(description: "identity bar laid out")
        measured.onChange = { height in
            if height > 0 { laidOut.fulfill() }
        }
        let root = VStack(spacing: 0) {
            WorkspaceIdentityBar(workspace: workspace)
                .background(
                    GeometryReader { geo in
                        Color.clear.onAppear {
                            measured.record(geo.size.height)
                        }
                        .onChange(of: geo.size.height) { _, height in
                            measured.record(height)
                        }
                    }
                )
            Spacer(minLength: 0)
        }
        .frame(width: 720, height: 640)

        let host = NSHostingController(rootView: root)
        let window = NSWindow(contentViewController: host)
        window.setContentSize(NSSize(width: 720, height: 640))
        window.isReleasedWhenClosed = false
        window.orderFrontRegardless()
        wait(for: [laidOut], timeout: 2)
        let height = measured.height
        window.close()

        XCTAssertGreaterThan(height, 16)
        XCTAssertLessThan(
            height,
            64,
            "Identity bar expanded to \(height)pt instead of hugging one line"
        )
    }
}

private final class IdentityBarHeightBox: @unchecked Sendable {
    var height: CGFloat = 0
    var onChange: ((CGFloat) -> Void)?
    private var reported = false

    func record(_ height: CGFloat) {
        self.height = height
        guard height > 0, !reported else { return }
        reported = true
        onChange?(height)
    }
}

@MainActor
final class WorkspaceSidebarViewModelTests: RepoViewModelTestCase {
    func testShowWorkspaceChangesUsesCommitIdsAndDoesNotEdit() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let dest = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-ws-\(UUID().uuidString)")
        try viewModel.repo.workspaceAdd(dest: dest.path, name: "feature", rev: "")
        let listed = try viewModel.repo.workspaceList()
        let feature = try XCTUnwrap(listed.first(where: { $0.name == "feature" }))
        XCTAssertFalse(feature.wcCommitId.isEmpty)
        XCTAssertFalse(feature.parentCommitId.isEmpty)

        let beforePath = viewModel.repoPath
        let beforeWC = try viewModel.repo.log(revset: "@").first?.commitId.id
        viewModel.showWorkspaceChanges(feature)

        for _ in 0 ..< 80 where viewModel.compareFromId == nil {
            try await Task.sleep(for: .milliseconds(25))
        }

        XCTAssertEqual(viewModel.repoPath, beforePath)
        XCTAssertEqual(try viewModel.repo.log(revset: "@").first?.commitId.id, beforeWC)
        XCTAssertEqual(viewModel.compareToId, feature.wcCommitId)
        XCTAssertFalse((viewModel.compareFromId ?? "").isEmpty)
        XCTAssertNotEqual(viewModel.compareFromId, "@")
        XCTAssertNotEqual(viewModel.compareToId, "@")
        try? FileManager.default.removeItem(at: dest)
    }

    func testAdoptMarksCurrentWithoutMovingSelectionOrPath() async throws {
        let viewModel = try XCTUnwrap(viewModel)
        let dest = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-ws-adopt-\(UUID().uuidString)")
        try viewModel.repo.workspaceAdd(dest: dest.path, name: "feature", rev: "")
        viewModel.refresh(snapshotWorkingCopy: false)
        var feature: WorkspaceInfo?
        for _ in 0 ..< 80 {
            feature = viewModel.workspaces.first(where: { $0.name == "feature" })
            if feature != nil { break }
            try await Task.sleep(for: .milliseconds(25))
        }
        let listed = try XCTUnwrap(feature)
        let beforePath = viewModel.repoPath
        let beforeSelection = viewModel.selectedChangeId
        let beforeGeneration = viewModel.workspaceSwitchGeneration

        viewModel.adoptWorkspaceAppearance(listed)

        XCTAssertEqual(viewModel.repoPath, beforePath)
        XCTAssertEqual(viewModel.selectedChangeId, beforeSelection)
        XCTAssertGreaterThan(viewModel.workspaceSwitchGeneration, beforeGeneration)
        XCTAssertEqual(viewModel.workspaces.first(where: \.isCurrent)?.name, "feature")
        XCTAssertEqual(
            WorkspaceSidebarPolicy.boundWorkspace(
                in: viewModel.workspaces,
                repoPath: viewModel.repoPath
            )?.name,
            "default"
        )
        try? FileManager.default.removeItem(at: dest)
    }

    func testCreateWorkspaceDoesNotChangeThisWindowUntilRebind() throws {
        let viewModel = try XCTUnwrap(viewModel)
        let dest = FileManager.default.temporaryDirectory
            .appending(path: "jayjay-ws-create-\(UUID().uuidString)")
        let before = viewModel.repoPath
        viewModel.workspaceAdd(dest: dest.path, name: "created")
        XCTAssertEqual(viewModel.repoPath, before)
        try? FileManager.default.removeItem(at: dest)
    }
}
