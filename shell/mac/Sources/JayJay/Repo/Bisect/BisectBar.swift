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
                markButton(Localized.Repo.good, verdict: .good, identifier: AID.Bisect.good)
                markButton(Localized.Repo.bad, verdict: .bad, identifier: AID.Bisect.bad)
                markButton(Localized.Repo.skip, verdict: .skip, identifier: AID.Bisect.skip)
                Button(Localized.Repo.runTestEllipsis) { showsTestRunner = true }
                    .buttonStyle(.bordered)
                    .disabled(busy)
                    .popover(isPresented: $showsTestRunner, arrowEdge: .bottom) {
                        testRunner(state, busy: busy)
                    }
                stopButton
            case .pausedRangeChanged:
                Button(Localized.Repo.restart) { viewModel.restartBisect() }
                    .buttonStyle(.bordered)
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.restart)
                stopButton
            case .finished:
                Button(Localized.Repo.dismiss) { viewModel.dismissBisect() }
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
        .help(Localized.Repo.stopBisectingAndRestoreTheWorking)
        .accessibilityLabel(Localized.Repo.stopBisecting)
        .accessibilityIdentifier(AID.Bisect.stop)
    }

    private func testRunner(_ state: BisectState, busy: Bool) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(Localized.Repo.exitN0MarksTheChangeGood)
                .jayjayFont(11)
                .foregroundStyle(.secondary)
            TextField(Localized.Repo.testCommand, text: $viewModel.bisect.command)
                .textFieldStyle(.roundedBorder)
                .frame(width: 320)
                .accessibilityIdentifier(AID.Bisect.command)
            HStack {
                Button(Localized.Repo.runForThisStep) { run(toEnd: false) }
                    .disabled(busy)
                    .accessibilityIdentifier(AID.Bisect.runStep)
                Button(Localized.Repo.runToTheEnd) { run(toEnd: true) }
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
            case .evaluating: Localized.Repo.bisecting
            case .pausedRangeChanged: Localized.Repo.bisectPaused
            case .finished: Localized.Repo.bisectFinished
        }
    }
}

private extension BisectResultView {
    var notes: [String] {
        (restoreNote.map { [$0] } ?? []) + kept.map { Localized.Repo.keptChange(description: $0.description.split(separator: "\n").first.map(String.init) ?? Localized.Repo.aChange) }
    }
}
