import JayJayCore
import SwiftUI

struct ShortcutSettingsRow: View {
    let binding: ShortcutBinding
    let recorder: ShortcutRecorder
    @State private var focus = ShortcutRecorderFocus.Target()
    @State private var hovered = false
    private static let cellWidth: CGFloat = 112
    private static let emptyWidth: CGFloat = 78

    private var active: Bool { recorder.commandID == binding.id }
    private var recording: Bool { active && recorder.isRecording }
    private var pending: Bool { active && recorder.pending != nil }
    private var editable: Bool { !binding.locked && !active }

    var body: some View {
        VStack(spacing: 4) {
            HStack(spacing: 6) {
                Text(binding.title).jayjayFont(12).foregroundStyle(binding.locked ? .secondary : .primary).frame(maxWidth: .infinity, alignment: .leading)
                Group {
                    if binding.modified {
                        Button {
                            if let focusView = focus.view { recorder.begin(id: binding.id, view: focusView) }
                            recorder.apply(.reset, id: binding.id)
                        } label: { Image(systemName: "arrow.uturn.backward") }
                            .buttonStyle(.plain)
                            .help(Localized.Shortcuts.reset)
                            .accessibilityIdentifier(AID.Shortcuts.reset(binding.id))
                    } else { Color.clear }
                }
                .frame(width: 20, height: 20)
                cell
            }
            if active, let message = recorder.message {
                HStack(spacing: 6) {
                    Text(message)
                        .jayjayFont(11)
                        .lineLimit(1)
                        .help(message)
                        .foregroundStyle(pending ? .orange : .red)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .accessibilityIdentifier(AID.Shortcuts.message)
                    if pending {
                        HStack {
                            if recorder.canReplace {
                                Button(Localized.Shortcuts.replace) { recorder.replace() }
                                    .frame(maxWidth: .infinity)
                                    .accessibilityIdentifier(AID.Shortcuts.replace)
                            }
                            Button(Localized.Common.cancel) { recorder.cancel() }.frame(maxWidth: .infinity)
                        }
                        .controlSize(.small)
                        .fixedSize()
                        .frame(minWidth: Self.cellWidth, alignment: .trailing)
                    }
                }
            }
        }
        .padding(.horizontal, 8)
        .padding(.vertical, 5)
        .overlay(alignment: .bottom) { Divider() }
    }

    private var cell: some View {
        Button {
            if let focusView = focus.view { recorder.begin(id: binding.id, view: focusView) }
        } label: {
            keys
                .fixedSize()
                .padding(.horizontal, 3)
                .frame(minHeight: 28)
                .background(editable && hovered ? Color.primary.opacity(0.06) : .clear, in: RoundedRectangle(cornerRadius: 5))
                .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .allowsHitTesting(editable)
        .onHover { hovered = $0 }
        .background(ShortcutRecorderFocus(target: focus))
        .overlay(RoundedRectangle(cornerRadius: 5).stroke(pending ? Color.orange : .clear))
        .frame(minWidth: Self.cellWidth, alignment: .trailing)
        .accessibilityLabel(binding.title)
        .accessibilityValue(binding.primary?.text ?? "")
        .accessibilityIdentifier(AID.Shortcuts.edit(binding.id))
    }

    private var keys: some View {
        Group {
            if recording {
                Group {
                    if recorder.heldCaps.isEmpty {
                        Text(Localized.Shortcuts.recordPrompt).jayjayFont(12).foregroundStyle(.secondary)
                    } else { ShortcutKeyCaps(chords: [recorder.heldCaps]) }
                }
                .padding(.horizontal, 3)
                .frame(minWidth: Self.emptyWidth, minHeight: 22)
                .overlay(RoundedRectangle(cornerRadius: 4).stroke(recorder.message == nil ? Color.accentColor : .red))
            } else if pending {
                ShortcutKeyCaps(chords: recorder.pendingChord.map { [$0.caps] } ?? [])
            } else if binding.chords.isEmpty {
                if !binding.locked {
                    Image(systemName: "plus")
                        .frame(width: Self.emptyWidth, height: 22)
                        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.secondary.opacity(0.4)))
                }
            } else {
                ShortcutKeyCaps(chords: binding.chords).foregroundStyle(binding.locked ? .secondary : .primary)
            }
        }
    }
}
