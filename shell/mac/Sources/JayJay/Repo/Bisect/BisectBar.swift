import JayJayCore
import SwiftUI

struct BisectBar: View {
    @Bindable var viewModel: RepoViewModel
    @State private var showsTestRunner = false

    var body: some View {
        if let state = viewModel.bisect.state {
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 10) {
                    Image(systemName: "point.topleft.down.to.point.bottomright.curvepath")
                        .foregroundStyle(.purple)
                    Text(state.phase.title)
                        .jayjayFont(12, weight: .semibold)
                    Text(state.status)
                        .jayjayFont(12)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                        .accessibilityIdentifier(AID.Bisect.status)
                    Spacer(minLength: 8)
                    actions(state)
                }
                if let error = state.commandError {
                    Text(error)
                        .jayjayFont(11)
                        .foregroundStyle(.red)
                        .accessibilityIdentifier(AID.Bisect.commandError)
                }
                if let result = state.result {
                    ForEach(result.notes, id: \.self) { note in
                        Text(note)
                            .jayjayFont(11)
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .controlSize(.small)
            .padding(.horizontal, PaneLayout.detailInset)
            .padding(.vertical, 8)
            .background(.purple.opacity(0.08))
            Divider()
        }
    }

    @ViewBuilder
    private func actions(_ state: BisectState) -> some View {
        let busy = !viewModel.bisect.isIdle || viewModel.isRefreshingInFlight
        switch state.phase {
            case .evaluating:
                markButton("Good", verdict: .good, identifier: AID.Bisect.good)
                markButton("Bad", verdict: .bad, identifier: AID.Bisect.bad)
                markButton("Skip", verdict: .skip, identifier: AID.Bisect.skip)
                Button("Run Test…") { showsTestRunner = true }
                    .buttonStyle(.bordered)
                    .disabled(busy)
                    .popover(isPresented: $showsTestRunner, arrowEdge: .bottom) {
                        testRunner(state, busy: busy)
                    }
                stopButton
            case .pausedRangeChanged:
                Button("Restart") { viewModel.restartBisect() }
                    .buttonStyle(.bordered)
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.restart)
                stopButton
            case .finished:
                Button("Dismiss") { viewModel.dismissBisect() }
                    .buttonStyle(.bordered)
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.dismiss)
        }
    }

    private var stopButton: some View {
        Button { viewModel.stopBisect() } label: {
            Image(systemName: "xmark.circle.fill")
                .foregroundStyle(.secondary)
        }
        .buttonStyle(.plain)
        .disabled(!viewModel.bisect.canStop || (viewModel.isRefreshingInFlight && !viewModel.bisect.commandIsRunning))
        .help("Stop bisecting and restore the working copy")
        .accessibilityLabel("Stop bisecting")
        .accessibilityIdentifier(AID.Bisect.stop)
    }

    private func testRunner(_ state: BisectState, busy: Bool) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Exit 0 marks the change good, 125 skips it, anything else marks it bad.")
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            TextField("Test command", text: $viewModel.bisect.command)
                .textFieldStyle(.roundedBorder)
                .frame(width: 320)
                .accessibilityIdentifier(AID.Bisect.command)
            HStack {
                Button("Run for This Step") { run(toEnd: false) }
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.runStep)
                Button("Run to the End") { run(toEnd: true) }
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.runToEnd)
            }
            if !state.commandOutput.isEmpty {
                Text(state.commandOutput)
                    .font(.system(size: 11, design: .monospaced))
                    .foregroundStyle(.secondary)
                    .lineLimit(6)
                    .textSelection(.enabled)
                    .accessibilityIdentifier(AID.Bisect.output)
            }
        }
        .padding(12)
    }

    private func run(toEnd: Bool) {
        showsTestRunner = false
        viewModel.runBisectCommand(toEnd: toEnd)
    }

    private func markButton(_ title: String, verdict: BisectVerdict, identifier: String) -> some View {
        Button(title) { viewModel.markBisect(verdict) }
            .buttonStyle(.bordered)
            .disabled(!viewModel.bisect.canMark || viewModel.isRefreshingInFlight)
            .accessibilityIdentifier(identifier)
    }
}

private extension BisectPhase {
    var title: String {
        switch self {
            case .evaluating: "Bisecting"
            case .pausedRangeChanged: "Bisect paused"
            case .finished: "Bisect finished"
        }
    }
}

private extension BisectResultView {
    var notes: [String] {
        (restoreNote.map { [$0] } ?? []) + kept.map { "Kept \($0.description.split(separator: "\n").first ?? "a change")" }
    }
}
