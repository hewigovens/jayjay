import SwiftUI

struct SyncArrowIndicator: View {
    enum Direction {
        case pull
        case push

        var label: String {
            switch self {
                case .pull: "Pull"
                case .push: "Push"
            }
        }

        var arrowSystemImage: String {
            switch self {
                case .pull: "arrow.down"
                case .push: "arrow.up"
            }
        }

        var sign: CGFloat {
            switch self {
                case .pull: 1
                case .push: -1
            }
        }
    }

    let direction: Direction
    let animating: Bool

    @Environment(\.accessibilityReduceMotion) private var reduceMotion
    @State private var startedAt = Date()

    private let duration = 0.7

    var body: some View {
        TimelineView(.animation(paused: !motionEnabled)) { context in
            let progress = motionEnabled ? progress(at: context.date) : 0
            Label {
                Text(direction.label)
            } icon: {
                ZStack {
                    Image(systemName: "circle")
                    Image(systemName: direction.arrowSystemImage)
                        .scaleEffect(0.55)
                        .offset(y: motionEnabled ? direction.sign * (-2 + 4 * progress) : 0)
                        .opacity(motionEnabled ? phaseOpacity(progress) : 1)
                }
            }
        }
        .onChange(of: motionEnabled, initial: true) { _, enabled in
            if enabled { startedAt = Date() }
        }
    }

    private var motionEnabled: Bool {
        animating && !reduceMotion
    }

    private func progress(at date: Date) -> CGFloat {
        CGFloat(date.timeIntervalSince(startedAt).truncatingRemainder(dividingBy: duration) / duration)
    }

    private func phaseOpacity(_ progress: CGFloat) -> Double {
        Double(min(min(progress * 5, (1 - progress) * 5), 1))
    }
}
