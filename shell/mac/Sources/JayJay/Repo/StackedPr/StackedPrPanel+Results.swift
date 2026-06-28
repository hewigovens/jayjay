import AppKit
import JayJayCore
import SwiftUI

extension StackedPrPanel {
    func resultsBody(_ result: StackedPrResult) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            ScrollView {
                VStack(spacing: 6) {
                    ForEach(result.layers.reversed(), id: \.bookmark) { layer in
                        resultRow(layer)
                    }
                }
            }
            .frame(maxHeight: 260)
            HStack {
                Spacer()
                Button("Done") {
                    openSubmittedPrs(result)
                    onDismiss()
                }
                .keyboardShortcut(.defaultAction)
                .buttonStyle(.borderedProminent)
                Spacer()
            }
        }
    }

    /// Open every created/updated PR's web page in the browser.
    private func openSubmittedPrs(_ result: StackedPrResult) {
        for layer in result.layers where !layer.prUrl.isEmpty {
            if let url = URL(string: layer.prUrl) {
                NSWorkspace.shared.open(url)
            }
        }
    }

    private func resultRow(_ layer: SubmittedLayer) -> some View {
        rowCard {
            Image(systemName: outcomeIcon(layer.outcome)).foregroundStyle(outcomeColor(layer.outcome))
            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 6) {
                    Text(layer.title.isEmpty ? layer.bookmark : layer.title)
                        .jayjayFont(12, weight: .medium).lineLimit(1)
                    if layer.prNumber > 0, let url = URL(string: layer.prUrl) {
                        Link("#\(layer.prNumber)", destination: url).jayjayFont(11, weight: .semibold)
                    }
                }
                Text(layer.detail).jayjayFont(10, design: .monospaced)
                    .foregroundStyle(.secondary).lineLimit(1)
            }
            Spacer()
        }
    }

    private func outcomeIcon(_ outcome: StackLayerOutcome) -> String {
        switch outcome {
            case .created: "plus.circle.fill"
            case .updated: "arrow.triangle.2.circlepath.circle.fill"
            case .failed: "xmark.octagon.fill"
        }
    }

    private func outcomeColor(_ outcome: StackLayerOutcome) -> Color {
        switch outcome {
            case .created: .green
            case .updated: .blue
            case .failed: .red
        }
    }
}
