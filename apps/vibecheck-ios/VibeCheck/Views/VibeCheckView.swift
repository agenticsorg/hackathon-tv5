import SwiftUI
import SwiftData

struct VibeCheckView: View {
    @Environment(\.modelContext) private var modelContext
    @Query(sort: \MoodLog.timestamp, order: .reverse) private var moodLogs: [MoodLog]
    @State private var healthManager = HealthKitManager()

    var body: some View {
        NavigationStack {
            List {
                currentReadingsSection
                activityLevelSection
                moodHistorySection
                privacySection
            }
            .navigationTitle("Vibe Check")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        Task {
                            await healthManager.fetchCurrentContext()
                        }
                    } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
            }
            .task {
                if healthManager.isHealthDataAvailable {
                    try? await healthManager.requestAuthorization()
                    await healthManager.fetchCurrentContext()
                }
            }
        }
    }

    private var currentReadingsSection: some View {
        Section("Current Readings") {
            ReadingRow(
                icon: "heart.text.square",
                iconColor: .red,
                label: "Heart Rate Variability",
                value: healthManager.currentHRV.map { String(format: "%.0f ms", $0) } ?? "—"
            )

            ReadingRow(
                icon: "bed.double",
                iconColor: .indigo,
                label: "Last Night's Sleep",
                value: healthManager.lastSleepHours.map { String(format: "%.1f hrs", $0) } ?? "—"
            )

            ReadingRow(
                icon: "figure.walk",
                iconColor: .green,
                label: "Steps Today",
                value: healthManager.stepsToday.map { String(format: "%.0f", $0) } ?? "—"
            )

            ReadingRow(
                icon: "heart",
                iconColor: .pink,
                label: "Resting Heart Rate",
                value: healthManager.restingHeartRate.map { String(format: "%.0f bpm", $0) } ?? "—"
            )
        }
    }

    private var activityLevelSection: some View {
        Section("Activity Level") {
            HStack {
                ForEach(HealthKitManager.ActivityLevel.allCases.filter { $0 != .unknown }, id: \.self) { level in
                    VStack(spacing: 4) {
                        Circle()
                            .fill(healthManager.activityLevel == level ? .green : Color.gray.opacity(0.3))
                            .frame(width: 12, height: 12)
                        Text(level.rawValue)
                            .font(.caption2)
                            .foregroundStyle(healthManager.activityLevel == level ? .primary : .secondary)
                    }
                    .frame(maxWidth: .infinity)
                }
            }
            .padding(.vertical, 8)
        }
    }

    private var moodHistorySection: some View {
        Section("Mood History") {
            if moodLogs.isEmpty {
                Text("No mood data yet")
                    .foregroundStyle(.secondary)
            } else {
                ForEach(moodLogs.prefix(10)) { log in
                    MoodLogRow(log: log)
                }
            }
        }
    }

    private var privacySection: some View {
        Section {
            HStack(spacing: 12) {
                Image(systemName: "lock.shield.fill")
                    .font(.title2)
                    .foregroundStyle(.green)

                VStack(alignment: .leading, spacing: 4) {
                    Text("Your Data Stays Private")
                        .font(.headline)
                    Text("All health data is processed on-device and never uploaded to any server.")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
            }
            .padding(.vertical, 8)
        }
    }
}

struct ReadingRow: View {
    let icon: String
    let iconColor: Color
    let label: String
    let value: String

    var body: some View {
        HStack {
            Image(systemName: icon)
                .font(.title2)
                .foregroundStyle(iconColor)
                .frame(width: 32)

            Text(label)
                .foregroundStyle(.secondary)

            Spacer()

            Text(value)
                .font(.headline)
                .monospacedDigit()
        }
    }
}

struct MoodLogRow: View {
    let log: MoodLog

    var body: some View {
        HStack {
            VStack(alignment: .leading, spacing: 4) {
                Text(log.recommendationHint.capitalized)
                    .font(.headline)

                HStack(spacing: 8) {
                    Text("Energy: \(log.energy.capitalized)")
                    Text("•")
                    Text("Stress: \(log.stress.capitalized)")
                }
                .font(.caption)
                .foregroundStyle(.secondary)
            }

            Spacer()

            Text(log.timestamp, style: .relative)
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
    }
}

#Preview {
    VibeCheckView()
        .modelContainer(for: [MoodLog.self], inMemory: true)
}
