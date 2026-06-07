@testable import JayJay
import XCTest

final class RefreshSpinnerTests: XCTestCase {
    private let degreesPerSecond = 360.0
    private let minSettleDuration: TimeInterval = 0.2

    private func params(from angle: Double) -> (target: Double, duration: Double) {
        RefreshSpinner.settleParams(from: angle, degreesPerSecond: degreesPerSecond, minSettleDuration: minSettleDuration)
    }

    // Each test sweeps angles from 0° to 1800° (five full rotations) in 7°
    // steps. 7 is coprime with 180 and 360, so it hits every distinct phase
    // within each half-turn rather than landing on the same offsets repeatedly.

    // MARK: - Target lands on a 180° multiple

    func testSettleTargetIsMultipleOf180() {
        for angle in stride(from: 0.0, through: 1800.0, by: 7.0) {
            let (target, _) = params(from: angle)
            XCTAssertEqual(target.truncatingRemainder(dividingBy: 180), 0, accuracy: 1e-9,
                           "target \(target) is not a 180° multiple for angle \(angle)")
        }
    }

    // MARK: - Target is always ahead of the starting angle

    func testSettleTargetIsAheadOfAngle() {
        for angle in stride(from: 0.0, through: 1800.0, by: 7.0) {
            let (target, _) = params(from: angle)
            XCTAssertGreaterThan(target, angle, "target should be past the starting angle")
        }
    }

    // MARK: - Duration is always positive

    func testSettleDurationIsPositive() {
        for angle in stride(from: 0.0, through: 1800.0, by: 7.0) {
            let (_, duration) = params(from: angle)
            XCTAssertGreaterThan(duration, 0, "settle duration must be positive")
        }
    }

    // MARK: - Settle starts at the same speed as the spin (no visible jerk)

    func testSettleStartsAtSpinSpeed() {
        // For cubic ease-out: v(0) = 3 × distance / duration.
        // This should equal degreesPerSecond.
        for angle in stride(from: 0.0, through: 1800.0, by: 7.0) {
            let (target, duration) = params(from: angle)
            let distance = target - angle
            let initialVelocity = 3 * distance / duration
            XCTAssertEqual(initialVelocity, degreesPerSecond, accuracy: 1e-9,
                           "initial settle velocity should match spin speed at angle \(angle)")
        }
    }

    // MARK: - Coast distance respects minimum

    func testCoastDistanceRespectsMinimum() {
        let minCoast = degreesPerSecond * minSettleDuration / 3
        for angle in stride(from: 0.0, through: 1800.0, by: 7.0) {
            let (target, _) = params(from: angle)
            XCTAssertGreaterThanOrEqual(target - angle, minCoast - 1e-9,
                                        "settle distance must be at least the minimum coast")
        }
    }
}
