import JayJayCore
import SwiftUI

struct SyncArrowIndicator: View {
    enum Direction {
        case pull
        case push

        var label: String {
            switch self {
                case .pull: Localized.Menu.pull
                case .push: Localized.Menu.push
            }
        }

        var arrowSystemImage: String {
            switch self {
                case .pull: "tray.and.arrow.down"
                case .push: "tray.and.arrow.up"
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
                Text(animating ? Localized.Repo.cancelDirection(direction: direction.label) : direction.label)
            } icon: {
                Image(systemName: direction.arrowSystemImage)
                    .opacity(motionEnabled ? 0.35 + 0.65 * phaseOpacity(progress) : 1)
            }
        }
        .onChange(of: motionEnabled, initial: true) { _, enabled in
            if enabled {
                startedAt = Date()
            }
        }
    }

    private var motionEnabled: Bool {
        animating && !reduceMotion
    }

    private func progress(at date: Date) -> CGFloat {
        let elapsed = max(0, date.timeIntervalSince(startedAt))
        return CGFloat(elapsed.truncatingRemainder(dividingBy: duration) / duration)
    }

    private func phaseOpacity(_ progress: CGFloat) -> Double {
        Double(min(min(progress * 5, (1 - progress) * 5), 1))
    }
}
