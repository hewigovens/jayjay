@testable import JayJay
import XCTest

final class AvatarStoreTests: XCTestCase {
    func testDiskURLUsesTheNativeApplicationCacheDirectory() throws {
        let cachesDirectory = try XCTUnwrap(
            FileManager.default.urls(for: .cachesDirectory, in: .userDomainMask).first
        )
        let key = AvatarStore.key("Person@example.com")

        XCTAssertEqual(
            AvatarStore.diskURL(key),
            cachesDirectory
                .appendingPathComponent("dev.hewig.jayjay", isDirectory: true)
                .appendingPathComponent("avatars", isDirectory: true)
                .appendingPathComponent("\(key).png")
        )
    }
}
