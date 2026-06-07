import SwiftUI

/// Refresh glyph that spins while `animating`, then decelerates to an upright stop.
struct RefreshSpinner: View {
    var animating: Bool

    /// Animation state: idle at rest, spinning at constant velocity while a
    /// refresh is in flight, then settling (decelerating to an upright stop).
    private enum Spin {
        case idle
        case spinning(since: Date)
        case settling(from: Double, target: Double, since: Date, duration: Double)
    }

    @State private var spin: Spin = .idle

    /// Spin speed while a refresh is in flight.
    private let degreesPerSecond = 360.0

    /// Floor for the settle phase; actual duration may be longer so the icon
    /// lands at a symmetry point with velocity-matched deceleration.
    private let minSettleDuration = 0.2

    /// Whether the timeline should stop ticking (no animation in flight).
    private var isResting: Bool {
        if case .idle = spin { return true }
        return false
    }

    var body: some View {
        TimelineView(.animation(paused: isResting)) { context in
            Label("Refresh", systemImage: "arrow.triangle.2.circlepath")
                .rotationEffect(.degrees(angle(at: context.date)))
        }
        .onChange(of: animating, initial: true) { _, active in
            if active {
                spin = .spinning(since: Date())
            } else if case let .spinning(since) = spin {
                beginSettle(from: Date().timeIntervalSince(since) * degreesPerSecond)
            }
        }
    }

    /// Current rotation angle in degrees, derived from the spin state and clock.
    private func angle(at date: Date) -> Double {
        switch spin {
            case .idle:
                return 0
            case let .spinning(since):
                return date.timeIntervalSince(since) * degreesPerSecond
            case let .settling(from, target, since, duration):
                let progress = min(1, date.timeIntervalSince(since) / duration)
                let eased = 1 - pow(1 - progress, 3) // ease-out cubic
                return from + (target - from) * eased
        }
    }

    /// Settle target and duration. The target is the nearest 180° multiple past
    /// a minimum coast distance, and the duration is derived so the cubic
    /// ease-out's initial velocity matches the spin speed.
    static func settleParams(
        from angle: Double,
        degreesPerSecond: Double,
        minSettleDuration: Double
    ) -> (target: Double, duration: Double) {
        // Minimum coast distance at the current spin velocity.
        let minCoast = degreesPerSecond * minSettleDuration / 3
        let natural = angle + minCoast
        // The symbol has 2-fold rotational symmetry, so 0° and 180° both look
        // upright. Target the nearest half-turn past the minimum coast point.
        let target = (natural / 180).rounded(.up) * 180
        let distance = target - angle
        let duration = 3 * distance / degreesPerSecond
        return (target, duration)
    }

    /// Transition from spinning to settling: captures the current angle and
    /// schedules a return to idle after the deceleration finishes.
    private func beginSettle(from angle: Double) {
        let startedAt = Date()
        let (target, duration) = Self.settleParams(
            from: angle, degreesPerSecond: degreesPerSecond, minSettleDuration: minSettleDuration
        )
        spin = .settling(from: angle, target: target, since: startedAt, duration: duration)
        DispatchQueue.main.asyncAfter(deadline: .now() + duration) {
            if case let .settling(_, _, since, _) = spin, since == startedAt {
                spin = .idle
            }
        }
    }
}
