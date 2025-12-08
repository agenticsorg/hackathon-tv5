import SwiftUI

struct VibeRing: View {
    let mood: MoodState
    @State private var animateRing = false

    var body: some View {
        ZStack {
            // Background ring
            Circle()
                .stroke(lineWidth: 20)
                .opacity(0.1)
                .foregroundStyle(AppTheme.accentGradient)

            // Energy ring (outer)
            Circle()
                .trim(from: 0, to: animateRing ? mood.energy.fillAmount : 0)
                .stroke(
                    energyGradient,
                    style: StrokeStyle(lineWidth: 20, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
                .animation(.spring(duration: 1.2, bounce: 0.3), value: animateRing)

            // Inner track
            Circle()
                .stroke(lineWidth: 12)
                .opacity(0.1)
                .foregroundStyle(.primary)
                .padding(24)

            // Stress ring (inner)
            Circle()
                .trim(from: 0, to: animateRing ? mood.stress.fillAmount : 0)
                .stroke(
                    stressGradient,
                    style: StrokeStyle(lineWidth: 12, lineCap: .round)
                )
                .rotationEffect(.degrees(-90))
                .padding(24)
                .animation(.spring(duration: 1.0, bounce: 0.3).delay(0.2), value: animateRing)

            // Center content
            VStack(spacing: 4) {
                Image(systemName: mood.moodIcon)
                    .font(.system(size: 36, weight: .medium))
                    .symbolRenderingMode(.hierarchical)
                    .foregroundStyle(energyGradient)

                Text(mood.recommendationHint.capitalized)
                    .font(.system(.headline, design: .rounded))
                    .fontWeight(.semibold)
                    .contentTransition(.numericText())

                Text("mode")
                    .font(.caption2)
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                    .tracking(1)
            }
        }
        .frame(width: 200, height: 200)
        .onAppear {
            animateRing = true
        }
        .onChange(of: mood) { _, _ in
            // Re-animate when mood changes
            animateRing = false
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                animateRing = true
            }
        }
        // Removed sensoryFeedback - may have issues on iOS 26 beta
    }

    private var energyGradient: LinearGradient {
        switch mood.energy {
        case .exhausted, .low:
            return LinearGradient(
                colors: [.purple, .indigo],
                startPoint: .leading,
                endPoint: .trailing
            )
        case .moderate:
            return LinearGradient(
                colors: [.green, .teal],
                startPoint: .leading,
                endPoint: .trailing
            )
        case .high, .wired:
            return LinearGradient(
                colors: [.orange, .yellow],
                startPoint: .leading,
                endPoint: .trailing
            )
        }
    }

    private var stressGradient: LinearGradient {
        switch mood.stress {
        case .relaxed, .calm:
            return LinearGradient(
                colors: [.mint, .teal],
                startPoint: .leading,
                endPoint: .trailing
            )
        case .neutral:
            return LinearGradient(
                colors: [.blue, .cyan],
                startPoint: .leading,
                endPoint: .trailing
            )
        case .tense, .stressed:
            return LinearGradient(
                colors: [.red, .orange],
                startPoint: .leading,
                endPoint: .trailing
            )
        }
    }
}

#Preview {
    VStack(spacing: 40) {
        VibeRing(mood: .chill)
        VibeRing(mood: .tired)
        VibeRing(mood: .energetic)
    }
    .padding()
}
