import SwiftUI

struct QuickMoodOverride: View {
    let onSelect: (MoodState) -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Not quite right?")
                .font(.subheadline)
                .foregroundStyle(.secondary)

            HStack(spacing: 10) {
                MoodOverrideButton(
                    label: "Tired",
                    icon: "moon.zzz.fill",
                    color: .purple
                ) {
                    onSelect(.tired)
                }

                MoodOverrideButton(
                    label: "Stressed",
                    icon: "bolt.heart.fill",
                    color: .red
                ) {
                    onSelect(.stressed)
                }

                MoodOverrideButton(
                    label: "Energetic",
                    icon: "figure.run",
                    color: .orange
                ) {
                    onSelect(.energetic)
                }

                MoodOverrideButton(
                    label: "Chill",
                    icon: "leaf.fill",
                    color: .green
                ) {
                    onSelect(.chill)
                }
            }
        }
    }
}

struct MoodOverrideButton: View {
    let label: String
    let icon: String
    let color: Color
    let action: () -> Void

    @State private var isPressed = false

    var body: some View {
        Button(action: action) {
            VStack(spacing: 8) {
                Image(systemName: icon)
                    .font(.title2)
                    .symbolRenderingMode(.hierarchical)
                    .foregroundStyle(color)

                Text(label)
                    .font(.caption2)
                    .fontWeight(.medium)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 14))
        }
        .buttonStyle(.plain)
        .scaleEffect(isPressed ? 0.95 : 1)
        .animation(.spring(duration: 0.2), value: isPressed)
        .sensoryFeedback(.selection, trigger: isPressed)
        .onLongPressGesture(minimumDuration: .infinity, maximumDistance: .infinity) {
        } onPressingChanged: { pressing in
            isPressed = pressing
        }
    }
}

#Preview {
    QuickMoodOverride { mood in
        print("Selected: \(mood.moodDescription)")
    }
    .padding()
}
