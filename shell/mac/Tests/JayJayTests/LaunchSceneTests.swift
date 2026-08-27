@testable import JayJay
import XCTest

final class LaunchSceneTests: XCTestCase {
    func testExternalToolSessionPresentsNoScene() {
        let scene = LaunchScene(isExternalTool: true, hasCompletedOnboarding: false, initialPath: "/repo")

        XCTAssertEqual(scene, .externalTool)
    }

    func testOnboardingRemembersTheLaunchPath() {
        let scene = LaunchScene(isExternalTool: false, hasCompletedOnboarding: false, initialPath: "/repo")

        XCTAssertEqual(scene, .onboarding(nextRepo: "/repo"))
        XCTAssertTrue(scene.repoPath.isEmpty, "The repository window must stay suppressed while onboarding shows")
    }

    func testOnboardingWithoutLaunchPathHasNoRepositoryToOpen() {
        XCTAssertEqual(
            LaunchScene(isExternalTool: false, hasCompletedOnboarding: false, initialPath: ""),
            .onboarding(nextRepo: nil)
        )
    }

    func testLaunchPathOpensItsRepository() {
        let scene = LaunchScene(isExternalTool: false, hasCompletedOnboarding: true, initialPath: "/repo")

        XCTAssertEqual(scene, .repo("/repo"))
        XCTAssertEqual(scene.repoPath, "/repo")
    }

    func testMissingLaunchPathOpensRepoList() {
        XCTAssertEqual(
            LaunchScene(isExternalTool: false, hasCompletedOnboarding: true, initialPath: nil),
            .repoList
        )
        XCTAssertEqual(
            LaunchScene(isExternalTool: false, hasCompletedOnboarding: true, initialPath: ""),
            .repoList
        )
    }
}
