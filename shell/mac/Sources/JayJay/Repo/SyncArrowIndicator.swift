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

        var imageName: String {
            switch self {
                case .pull: "sync.pull"
                case .push: "sync.push"
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
    @State private var finished = 0

    var body: some View {
        Label {
            Text(animating ? Localized.Repo.cancelDirection(direction: direction.label) : direction.label)
        } icon: {
            Image(direction.imageName)
                .resizable()
                .frame(width: 19.3, height: 16)
                .keyframeAnimator(initialValue: 0, repeating: animating && !reduceMotion) { icon, nudge in
                    icon.offset(y: direction.sign * nudge)
                } keyframes: { _ in
                    CubicKeyframe(1.5, duration: 0.35)
                    CubicKeyframe(0, duration: 0.35)
                    CubicKeyframe(0, duration: 0.2)
                }
                .keyframeAnimator(initialValue: 0, trigger: finished) { icon, bounce in
                    icon.offset(y: direction.sign * bounce)
                } keyframes: { _ in
                    SpringKeyframe(-2, duration: 0.12, spring: .snappy)
                    SpringKeyframe(0, duration: 0.4, spring: .bouncy)
                }
        }
        .onChange(of: animating) { wasAnimating, isAnimating in
            if wasAnimating, !isAnimating, !reduceMotion {
                finished += 1
            }
        }
    }
}
