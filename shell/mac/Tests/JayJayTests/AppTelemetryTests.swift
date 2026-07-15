import Foundation
@testable import JayJay
import XCTest

final class AppTelemetryTests: XCTestCase {
    func testDailyAndMonthlyIDsRotateAtTheirUTCBoundaries() throws {
        let formatter = ISO8601DateFormatter()
        let first = try XCTUnwrap(formatter.date(from: "2026-07-14T23:59:59Z"))
        let nextDay = try XCTUnwrap(formatter.date(from: "2026-07-15T00:00:00Z"))
        let nextMonth = try XCTUnwrap(formatter.date(from: "2026-08-01T00:00:00Z"))
        let secret = "local-install-secret"

        let firstPeriods = AppTelemetry.periods(at: first)
        let nextDayPeriods = AppTelemetry.periods(at: nextDay)
        let nextMonthPeriods = AppTelemetry.periods(at: nextMonth)

        XCTAssertNotEqual(
            AppTelemetry.periodID(secret: secret, scope: "day", period: firstPeriods.day),
            AppTelemetry.periodID(secret: secret, scope: "day", period: nextDayPeriods.day)
        )
        XCTAssertEqual(
            AppTelemetry.periodID(secret: secret, scope: "month", period: firstPeriods.month),
            AppTelemetry.periodID(secret: secret, scope: "month", period: nextDayPeriods.month)
        )
        XCTAssertNotEqual(
            AppTelemetry.periodID(secret: secret, scope: "month", period: firstPeriods.month),
            AppTelemetry.periodID(secret: secret, scope: "month", period: nextMonthPeriods.month)
        )
    }
}
