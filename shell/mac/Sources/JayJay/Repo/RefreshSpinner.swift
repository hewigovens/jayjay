import SwiftUI

struct RefreshSpinner: View {
    var animating: Bool
    @Environment(\.accessibilityReduceMotion) private var reduceMotion

    private enum Spin {
        case idle
        case spinning(since: Date)
        case settling(from: Double, target: Double, since: Date, duration: Double)
    }

    @State private var spin: Spin = .idle

    private let degreesPerSecond = 360.0
    private let minSettleDuration = 0.2

    private var isResting: Bool {
        if case .idle = spin { return true }
        return false
    }

    var body: some View {
        TimelineView(.animation(paused: isResting || reduceMotion)) { context in
            Label("Refresh", systemImage: "arrow.triangle.2.circlepath")
                .rotationEffect(.degrees(reduceMotion ? 0 : angle(at: context.date)))
        }
        .onChange(of: animating, initial: true) { _, active in
            if active {
                spin = .spinning(since: Date())
            } else if case let .spinning(since) = spin {
                beginSettle(from: Date().timeIntervalSince(since) * degreesPerSecond)
            }
        }
    }

    private func angle(at date: Date) -> Double {
        switch spin {
            case .idle:
                return 0
            case let .spinning(since):
                return date.timeIntervalSince(since) * degreesPerSecond
            case let .settling(from, target, since, duration):
                let progress = min(1, date.timeIntervalSince(since) / duration)
                let eased = 1 - pow(1 - progress, 3)
                return from + (target - from) * eased
        }
    }

    static func settleParams(
        from angle: Double,
        degreesPerSecond: Double,
        minSettleDuration: Double
    ) -> (target: Double, duration: Double) {
        let minCoast = degreesPerSecond * minSettleDuration / 3
        let natural = angle + minCoast
        // The symbol has 2-fold rotational symmetry, so 0° and 180° both look upright.
        let target = (natural / 180).rounded(.up) * 180
        let distance = target - angle
        let duration = 3 * distance / degreesPerSecond
        return (target, duration)
    }

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
