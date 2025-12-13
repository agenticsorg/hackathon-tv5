import SwiftUI

struct VibeHeader: View {
    let mood: MoodState?
    let isLoading: Bool

    var body: some View {
        VStack(spacing: 8) {
            if isLoading {
                ProgressView()
                    .controlSize(.regular)

                Text("Checking your vibe...")
                    .font(.title3)
                    .foregroundStyle(.secondary)
            } else if let mood = mood {
                Text("You seem")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)

                Text(mood.moodDescription)
                    .font(.system(size: 34, weight: .bold, design: .rounded))
                    .contentTransition(.numericText())

                // Confidence indicator
                if mood.confidence < 0.5 {
                    HStack(spacing: 4) {
                        Image(systemName: "questionmark.circle")
                        Text("Limited health data")
                    }
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.top, 4)
                }
            } else {
                Text("Ready to check in")
                    .font(.title2)
                    .foregroundStyle(.secondary)
            }
        }
        .animation(.spring, value: mood)
        .animation(.spring, value: isLoading)
    }
}

#Preview {
    VStack(spacing: 40) {
        VibeHeader(mood: nil, isLoading: true)
        VibeHeader(mood: .chill, isLoading: false)
        VibeHeader(mood: .tired, isLoading: false)
    }
}
