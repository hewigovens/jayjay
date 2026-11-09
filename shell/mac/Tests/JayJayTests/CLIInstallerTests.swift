@testable import JayJay
import XCTest

final class CLIInstallerTests: XCTestCase {
    func testOnlyLinksIntoAnAppBundleAreManaged() throws {
        let fm = FileManager.default
        let dir = fm.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try fm.createDirectory(at: dir, withIntermediateDirectories: true)
        defer { try? fm.removeItem(at: dir) }

        let managed = dir.appendingPathComponent("managed").path
        try fm.createSymbolicLink(atPath: managed, withDestinationPath: "/Applications/Old JayJay.app/Contents/MacOS/jayjay-cli")
        let foreignLink = dir.appendingPathComponent("foreign-link").path
        try fm.createSymbolicLink(atPath: foreignLink, withDestinationPath: "/usr/local/bin/jayjay")
        let script = dir.appendingPathComponent("script").path
        fm.createFile(atPath: script, contents: Data("#!/bin/sh\n".utf8))

        XCTAssertTrue(CLIInstaller.isManagedLink(at: managed))
        XCTAssertFalse(CLIInstaller.isManagedLink(at: foreignLink))
        XCTAssertFalse(CLIInstaller.isManagedLink(at: script))
        XCTAssertFalse(CLIInstaller.isManagedLink(at: dir.appendingPathComponent("missing").path))
    }
}
